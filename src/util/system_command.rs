use crate::error::{Error, Result};
use std::ffi::OsStr;
use std::io::{ErrorKind, Read};
use std::process::{Command, Output, Stdio};
use std::str::from_utf8;

pub struct SystemCommand {
    cmd: Command,
    stdout: bool,
}

impl SystemCommand {
    pub fn new(program: &str) -> Self {
        Self {
            cmd: Command::new(program),
            stdout: false,
        }
    }

    pub fn get_command(&self) -> String {
        format!(
            "{} {}",
            self.cmd.get_program().to_str().unwrap(),
            self.cmd
                .get_args()
                .map(|a| a.to_str().unwrap())
                .collect::<Vec<_>>()
                .join(" ")
        )
        .trim()
        .to_string()
    }

    fn get_program(&self) -> String {
        self.cmd.get_program().to_string_lossy().into_owned()
    }

    fn map_spawn_error(&self, error: std::io::Error) -> Error {
        if error.kind() == ErrorKind::NotFound {
            let program = self.get_program();
            if program == "qemu-img" || program.ends_with("/qemu-img") {
                return Error::QemuImgNotFound;
            }
            if program.starts_with("qemu-") || program.contains("/qemu-") {
                return Error::QemuNotFound(program);
            }

            return Error::SystemCommandNotFound(program);
        }

        Error::SystemCommandFailed(self.get_command(), error.to_string())
    }

    pub fn env(&mut self, key: &str, value: &str) -> &mut Self {
        self.cmd.env(key, value);
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.cmd.args(args);
        self
    }

    pub fn run(&mut self) -> Result<()> {
        self.cmd
            .stdin(Stdio::null())
            .stdout(if self.stdout {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| self.map_spawn_error(e))
            .and_then(|out| {
                if out.status.success() {
                    Ok(())
                } else {
                    Err(Error::SystemCommandFailed(
                        self.get_command(),
                        from_utf8(&out.stderr).unwrap_or_default().to_string(),
                    ))
                }
            })
    }

    pub fn run_daemonized(&mut self) -> Result<()> {
        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt;
            self.cmd.pre_exec(detach_from_session);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            self.cmd.creation_flags(DETACHED_PROCESS);
        }

        let mut child = self
            .cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| self.map_spawn_error(e))?;

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
                return Err(Error::SystemCommandFailed(self.get_command(), stderr));
            }
        }

        // Reap the child when it eventually exits to avoid a zombie process.
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }

    pub fn output(&mut self) -> Result<Output> {
        self.cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| self.map_spawn_error(e))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command() {
        assert_eq!(SystemCommand::new("cubic").get_command(), "cubic")
    }

    #[test]
    fn test_get_command_with_one_arg() {
        assert_eq!(
            SystemCommand::new("cubic").arg("-a").get_command(),
            "cubic -a"
        )
    }

    #[test]
    fn test_get_command_with_two_args() {
        assert_eq!(
            SystemCommand::new("cubic")
                .arg("-a")
                .arg("-b")
                .get_command(),
            "cubic -a -b"
        )
    }

    #[test]
    fn test_run_reports_missing_qemu_binary() {
        let program = "qemu-system-x86_64-cubic-missing-test";

        let err = SystemCommand::new(program).run().unwrap_err();

        assert!(matches!(err, Error::QemuNotFound(ref missing) if missing == program));
        let message = err.to_string();
        assert!(message.contains("QEMU not found"));
        assert!(message.contains("brew install qemu"));
    }

    #[test]
    fn test_output_reports_missing_qemu_binary() {
        let program = "qemu-img-cubic-missing-test";

        let err = SystemCommand::new(program).output().unwrap_err();

        assert!(matches!(err, Error::QemuNotFound(ref missing) if missing == program));
    }

    #[test]
    fn test_output_reports_missing_qemu_img_binary() {
        let err = SystemCommand::new("/nonexistent/path/qemu-img")
            .output()
            .unwrap_err();

        assert!(matches!(err, Error::QemuImgNotFound));
        let message = err.to_string();
        assert!(message.contains("qemu-img not found"));
        assert!(message.contains("CUBIC_QEMU_IMG"));
    }

    #[test]
    fn test_output_reports_missing_system_command() {
        let program = "cubic-missing-command-test";

        let err = SystemCommand::new(program).output().unwrap_err();

        assert!(matches!(err, Error::SystemCommandNotFound(ref missing) if missing == program));
    }
}
