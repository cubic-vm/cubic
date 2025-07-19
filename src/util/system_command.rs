use crate::error::Error;
use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};
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

    pub fn set_stdout(&mut self, stdout: bool) -> &mut Self {
        self.stdout = stdout;
        self
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

    pub fn run(&mut self) -> Result<(), Error> {
        self.cmd
            .stdin(Stdio::null())
            .stdout(if self.stdout {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Error::SystemCommandFailed(self.get_command(), e.to_string()))
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

    pub fn spawn(&mut self) -> Result<Child, Error> {
        self.cmd
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::SystemCommandFailed(self.get_command(), e.to_string()))
    }
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
}
