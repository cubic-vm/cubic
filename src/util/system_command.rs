use std::ffi::{OsStr, OsString};

// A command to run, described but not started. Turning one into a child
// process is the job of the `System` implementation, which keeps every spawn
// on the host behind the same seam as every file access.
pub struct SystemCommand {
    program: String,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
}

impl SystemCommand {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
            envs: Vec::new(),
        }
    }

    pub fn get_command(&self) -> String {
        format!(
            "{} {}",
            self.program,
            self.args
                .iter()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        )
        .trim()
        .to_string()
    }

    pub fn get_program(&self) -> &str {
        &self.program
    }

    pub fn get_args(&self) -> &[OsString] {
        &self.args
    }

    pub fn get_envs(&self) -> &[(OsString, OsString)] {
        &self.envs
    }

    pub fn set_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(&mut self, key: K, value: V) -> &mut Self {
        self.envs
            .push((key.as_ref().to_os_string(), value.as_ref().to_os_string()));
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
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
