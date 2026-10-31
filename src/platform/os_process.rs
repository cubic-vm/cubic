use crate::error::{Error, Result};
use crate::platform::{OsSystem, Process};
use crate::util::CommandLine;
use std::io::{ErrorKind, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
    fn run_and_wait(
        &self,
        mut command: Command,
        stdin: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Self::map_spawn_error(&command, e))?;

        // Dropping the pipe sends EOF.
        if let (Some(mut pipe), Some(input)) = (child.stdin.take(), stdin) {
            let _ = pipe.write_all(input.as_bytes());
        }
        // Output is not drained while polling, so only for commands that print little.
        if let Some(timeout) = timeout {
            let deadline = Instant::now() + timeout;
            while let Ok(None) = child.try_wait() {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(Error::SystemCommandFailed(
                        command.get_command(),
                        "The command timed out.".to_string(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let out = child
            .wait_with_output()
            .map_err(|e| Error::SystemCommandFailed(command.get_command(), e.to_string()))?;

        if out.status.success() {
            Ok(out.stdout)
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
            Err(Error::SystemCommandFailed(command.get_command(), stderr))
        }
    }

    fn run_detached(&self, mut command: Command) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt;
            command.pre_exec(detach_from_session);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            command.creation_flags(DETACHED_PROCESS);
        }

        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Self::map_spawn_error(&command, e))?;

        // Check immediately for instant failures, then again after a brief window
        // to catch startup errors (KVM permission denied, bad firmware, port conflicts).
        for ms in [0, 100] {
            std::thread::sleep(Duration::from_millis(ms));
            if let Ok(Some(_)) = child.try_wait() {
                let stderr = child
                    .wait_with_output()
                    .map(|o| String::from_utf8_lossy(&o.stderr).into_owned())
                    .unwrap_or_default();
                return Err(Error::SystemCommandFailed(command.get_command(), stderr));
            }
        }

        // Drains stderr and reaps the child.
        std::thread::spawn(move || {
            if let Some(mut stderr) = child.stderr.take() {
                let _ = std::io::copy(&mut stderr, &mut std::io::sink());
            }
            let _ = child.wait();
        });

        Ok(())
    }
}
