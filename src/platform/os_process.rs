use crate::error::{Error, Result};
use crate::platform::{OsSystem, Process};
use crate::util::SystemCommand;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::str::from_utf8;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

impl OsSystem {
    // Refreshes a single process so a lookup sees the current state of the
    // host. `sysinfo::System` is spelled out because the name collides with
    // the `System` trait of this crate.
    fn read_process_table(pid: u64) -> (sysinfo::System, sysinfo::Pid) {
        let sys_pid = sysinfo::Pid::from_u32(pid as u32);
        let mut system = sysinfo::System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);
        (system, sys_pid)
    }

    fn build_process(command: &SystemCommand) -> Command {
        let mut process = Command::new(command.get_program());
        process.args(command.get_args());
        for (key, value) in command.get_envs() {
            process.env(key, value);
        }
        process
    }

    // A missing binary is worth telling apart from a binary that refused to
    // start, since only the first one means the host lacks the tool.
    fn map_spawn_error(command: &SystemCommand, error: std::io::Error) -> Error {
        if error.kind() == ErrorKind::NotFound {
            return Error::SystemCommandNotFound(command.get_program().to_string());
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
    fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>> {
        Self::build_process(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Self::map_spawn_error(command, e))
            .and_then(|out| {
                if out.status.success() {
                    Ok(out.stdout)
                } else {
                    Err(Error::SystemCommandFailed(
                        command.get_command(),
                        from_utf8(&out.stderr).unwrap_or_default().to_string(),
                    ))
                }
            })
    }

    // Reads rather than polls, so the command is only kept alive until it says
    // it is up. Stdin stays open, or a command that reads it sees an early end
    // of file and stops on its own.
    fn run_command_until_output(
        &self,
        command: &SystemCommand,
        marker: &str,
        timeout: Duration,
    ) -> Result<()> {
        let mut child = Self::build_process(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Self::map_spawn_error(command, e))?;

        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            let _ = child.kill();
            return Err(Error::SystemCommandFailed(
                command.get_command(),
                "The command offered no output to read.".to_string(),
            ));
        };

        // Both pipes are drained on their own thread. A pipe that nobody reads
        // fills up and stops the command it belongs to.
        let (sender, receiver) = mpsc::channel();
        let needle = marker.to_string();
        thread::spawn(move || {
            let mut lines = BufReader::new(stdout).lines();
            while let Some(Ok(line)) = lines.next() {
                if line.contains(&needle) {
                    return sender.send(()).unwrap_or_default();
                }
            }
        });
        let collector = thread::spawn(move || {
            let mut text = String::new();
            BufReader::new(stderr).read_to_string(&mut text).ok();
            text
        });

        // The sender is dropped as soon as stdout ends, so a command that stops
        // early is seen without waiting for the deadline.
        let found = receiver.recv_timeout(timeout).is_ok();
        let _ = child.kill();
        let _ = child.wait();

        if found {
            return Ok(());
        }
        Err(Error::SystemCommandFailed(
            command.get_command(),
            collector.join().unwrap_or_default(),
        ))
    }

    fn spawn_command(&self, command: &SystemCommand) -> Result<()> {
        let mut process = Self::build_process(command);

        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt;
            process.pre_exec(detach_from_session);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            process.creation_flags(DETACHED_PROCESS);
        }

        let mut child = process
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Self::map_spawn_error(command, e))?;

        // Check immediately for instant failures, then again after a brief window
        // to catch startup errors (KVM permission denied, bad firmware, port conflicts).
        for ms in [0, 100] {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            if let Ok(Some(_)) = child.try_wait() {
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).unwrap_or(0);
                        from_utf8(&buf).unwrap_or_default().to_owned()
                    })
                    .unwrap_or_default();
                return Err(Error::SystemCommandFailed(command.get_command(), stderr));
            }
        }

        // Reap the child when it eventually exits to avoid a zombie process.
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }

    fn exists_process(&self, pid: u64) -> bool {
        let (system, sys_pid) = Self::read_process_table(pid);
        system.process(sys_pid).is_some()
    }

    fn kill_process(&self, pid: u64) -> Result<()> {
        let (system, sys_pid) = Self::read_process_table(pid);
        let process = system.process(sys_pid).ok_or(Error::ProcessNotFound(pid))?;

        process.kill().then_some(()).ok_or(Error::KillFailed(pid))
    }
}
