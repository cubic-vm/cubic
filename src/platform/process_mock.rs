use crate::error::{Error, Result};
use crate::platform::{Child, ChildExit, ChildInput, ChildOutput, Process, SystemMock};
use crate::util::CommandLine;
use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;
use tokio::io;

// How a seeded command answers a run.
enum CommandResult {
    // Exits successfully with this on stdout.
    Output(Vec<u8>),
    // Exits with a failure carrying this on stderr.
    Failure(String),
}

// The commands the host knows how to answer, alongside the record of what was
// attempted. A command that is not seeded is one the host cannot run, so it
// reports a missing binary rather than quietly succeeding.
#[derive(Default)]
pub struct CommandMock {
    results: HashMap<String, CommandResult>,
    executed: Vec<String>,
}

impl CommandMock {
    fn add(&mut self, command: &str, result: CommandResult) {
        self.results.insert(command.to_string(), result);
    }

    fn get_executed(&self) -> Vec<String> {
        self.executed.clone()
    }

    fn spawn(&mut self, command: &Command) -> Result<ChildMock> {
        let line = command.get_command();
        self.executed.push(line.clone());

        match self.results.get(line.as_str()) {
            None => Err(Error::SystemCommandNotFound(
                command.get_program().to_string_lossy().into_owned(),
            )),
            Some(CommandResult::Output(stdout)) => Ok(ChildMock {
                stdout: stdout.clone(),
                success: true,
                ..ChildMock::default()
            }),
            Some(CommandResult::Failure(stderr)) => Ok(ChildMock {
                stderr: stderr.clone().into_bytes(),
                success: false,
                ..ChildMock::default()
            }),
        }
    }
}

impl SystemMock {
    pub fn add_command_output(self, command: &str, stdout: &[u8]) -> Self {
        self.insert_command_output(command, stdout);
        self
    }

    pub fn insert_command_output(&self, command: &str, stdout: &[u8]) {
        self.commands
            .lock()
            .unwrap()
            .add(command, CommandResult::Output(stdout.to_vec()));
    }

    pub fn add_failing_command(self, command: &str, stderr: &str) -> Self {
        self.commands
            .lock()
            .unwrap()
            .add(command, CommandResult::Failure(stderr.to_string()));
        self
    }

    // Every command the host was asked to run, seeded or not, in order.
    pub fn get_executed_commands(&self) -> Vec<String> {
        self.commands.lock().unwrap().get_executed()
    }
}

#[derive(Default)]
struct ChildMock {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    success: bool,
}

impl Child for ChildMock {
    fn wait(&mut self) -> ChildExit<'_> {
        Box::pin(std::future::ready(self.success))
    }

    fn take_stdin(&mut self) -> ChildInput {
        Box::new(io::sink())
    }

    fn take_stdout(&mut self) -> ChildOutput {
        Box::new(Cursor::new(std::mem::take(&mut self.stdout)))
    }

    fn take_stderr(&mut self) -> ChildOutput {
        Box::new(Cursor::new(std::mem::take(&mut self.stderr)))
    }
}

impl Process for SystemMock {
    fn spawn(&self, command: Command, _detach: bool) -> Result<Box<dyn Child>> {
        let child = self.commands.lock().unwrap().spawn(&command)?;
        Ok(Box::new(child))
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::{Process, SystemMock};
    use std::process::Command;

    #[test]
    fn spawn_matches_on_the_arguments() {
        let system = SystemMock::new().add_command_output("qemu-img info a", b"a");

        let mut other = Command::new("qemu-img");
        other.arg("info").arg("b");

        assert!(system.spawn(other, false).is_err());
    }

    #[test]
    fn get_executed_commands_records_every_attempt_in_order() {
        let system = SystemMock::new().add_command_output("echo one", b"");

        let mut first = Command::new("echo");
        first.arg("one");
        system.spawn(first, false).unwrap();

        let mut second = Command::new("echo");
        second.arg("two");
        system.spawn(second, false).ok();

        // The failed attempt is recorded too, since the host was still asked
        // to run it.
        assert_eq!(system.get_executed_commands(), vec!["echo one", "echo two"]);
    }
}
