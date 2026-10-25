use std::process::Command;

// The command as one line, for errors, logs and the mock.
pub trait CommandLine {
    fn get_command(&self) -> String;
}

impl CommandLine for Command {
    fn get_command(&self) -> String {
        std::iter::once(self.get_program())
            .chain(self.get_args())
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_command() {
        assert_eq!(Command::new("cubic").get_command(), "cubic")
    }

    #[test]
    fn test_get_command_with_two_args() {
        assert_eq!(
            Command::new("cubic").arg("-a").arg("-b").get_command(),
            "cubic -a -b"
        )
    }
}
