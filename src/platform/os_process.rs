use crate::error::{Error, Result};
use crate::platform::{Child, ChildExit, ChildInput, ChildOutput, OsSystem, Process};
use crate::util::CommandLine;
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use tokio::io;

struct OsChild {
    child: tokio::process::Child,
}

impl Child for OsChild {
    fn wait(&mut self) -> ChildExit<'_> {
        Box::pin(async move { self.child.wait().await.is_ok_and(|status| status.success()) })
    }

    fn take_stdin(&mut self) -> ChildInput {
        match self.child.stdin.take() {
            Some(pipe) => Box::new(pipe),
            None => Box::new(io::sink()),
        }
    }

    fn take_stdout(&mut self) -> ChildOutput {
        match self.child.stdout.take() {
            Some(pipe) => Box::new(pipe),
            None => Box::new(io::empty()),
        }
    }

    fn take_stderr(&mut self) -> ChildOutput {
        match self.child.stderr.take() {
            Some(pipe) => Box::new(pipe),
            None => Box::new(io::empty()),
        }
    }
}

impl OsSystem {
    // A missing binary is worth telling apart from a binary that refused to
    // start, since only the first one means the host lacks the tool.
    fn map_spawn_error(command: &Command, error: std::io::Error) -> Error {
        if error.kind() == ErrorKind::NotFound {
            let program = command.get_program().to_string_lossy().into_owned();
            return Error::SystemCommandNotFound(program);
        }

        Error::SystemCommandFailed(command.get_command(), error.to_string())
    }
}

// Detaches the child from the session of the parent, so it survives the shell
// that started cubic.
#[cfg(unix)]
fn detach_from_session() -> std::io::Result<()> {
    unsafe extern "C" {
        fn setsid() -> i32;
    }
    if unsafe { setsid() } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

impl Process for OsSystem {
    fn spawn(&self, mut command: Command, detach: bool) -> Result<Box<dyn Child>> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            if detach {
                unsafe { command.pre_exec(detach_from_session) };
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            if detach {
                command.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
            }
        }

        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut command = tokio::process::Command::from(command);
        command.kill_on_drop(!detach);
        let child = command
            .spawn()
            .map_err(|e| Self::map_spawn_error(command.as_std(), e))?;

        Ok(Box::new(OsChild { child }))
    }
}
