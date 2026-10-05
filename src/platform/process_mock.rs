use crate::error::{Error, Result};
use crate::platform::{Process, SystemMock};
use crate::util::SystemCommand;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

// The pids the host knows about.
#[derive(Default)]
pub struct ProcessMock {
    processes: HashSet<u64>,
}

impl ProcessMock {
    fn add(&mut self, pid: u64) {
        self.processes.insert(pid);
    }

    fn exists(&self, pid: u64) -> bool {
        self.processes.contains(&pid)
    }
}

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

    fn run(&mut self, command: &SystemCommand) -> Result<Vec<u8>> {
        let line = command.get_command();
        self.executed.push(line.clone());

        match self.results.get(line.as_str()) {
            None => Err(Error::SystemCommandNotFound(
                command.get_program().to_string(),
            )),
            Some(CommandResult::Failure(stderr)) => {
                Err(Error::SystemCommandFailed(line, stderr.clone()))
            }
            Some(CommandResult::Output(stdout)) => Ok(stdout.clone()),
        }
    }
}

impl SystemMock {
    pub fn add_process(self, pid: u64) -> Self {
        self.processes.lock().unwrap().add(pid);
        self
    }

    pub fn add_command_output(self, command: &str, stdout: &[u8]) -> Self {
        self.commands
            .lock()
            .unwrap()
            .add(command, CommandResult::Output(stdout.to_vec()));
        self
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

impl Process for SystemMock {
    fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>> {
        self.commands.lock().unwrap().run(command)
    }

    // A seeded command stands for one that comes up and prints the marker, so
    // neither the marker nor the deadline matters here.
    fn run_command_until_output(
        &self,
        command: &SystemCommand,
        _marker: &str,
        _timeout: Duration,
    ) -> Result<()> {
        self.commands.lock().unwrap().run(command).map(|_| ())
    }

    // A detached start has nothing to wait for, so it only reports whether the
    // host could launch the command at all.
    fn spawn_command(&self, command: &SystemCommand) -> Result<()> {
        self.commands.lock().unwrap().run(command).map(|_| ())
    }

    fn exists_process(&self, pid: u64) -> bool {
        self.processes.lock().unwrap().exists(pid)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::platform::{Process, SystemMock};
    use crate::util::SystemCommand;

    #[test]
    fn exists_process_only_finds_seeded_pids() {
        let system = SystemMock::new().add_process(42);

        assert!(system.exists_process(42));
        assert!(!system.exists_process(43));
    }

    #[test]
    fn run_command_returns_the_seeded_output() {
        let system = SystemMock::new().add_command_output("echo hello", b"hello\n");

        let mut command = SystemCommand::new("echo");
        command.arg("hello");

        assert_eq!(system.run_command(&command).unwrap(), b"hello\n");
    }

    #[test]
    fn run_command_fails_for_an_unseeded_command() {
        let system = SystemMock::new();

        assert!(matches!(
            system.run_command(&SystemCommand::new("qemu-img")),
            Err(Error::SystemCommandNotFound(program)) if program == "qemu-img"
        ));
    }

    #[test]
    fn run_command_reports_a_seeded_failure_with_its_stderr() {
        let system = SystemMock::new().add_failing_command("qemu-img resize", "no such file");

        let mut command = SystemCommand::new("qemu-img");
        command.arg("resize");

        assert!(matches!(
            system.run_command(&command),
            Err(Error::SystemCommandFailed(line, stderr))
                if line == "qemu-img resize" && stderr == "no such file"
        ));
    }

    #[test]
    fn run_command_matches_on_the_arguments() {
        let system = SystemMock::new().add_command_output("qemu-img info a", b"a");

        let mut other = SystemCommand::new("qemu-img");
        other.arg("info").arg("b");

        assert!(system.run_command(&other).is_err());
    }

    #[test]
    fn spawn_command_succeeds_without_returning_output() {
        let system = SystemMock::new().add_command_output("qemu-system-x86_64", b"ignored");

        system
            .spawn_command(&SystemCommand::new("qemu-system-x86_64"))
            .unwrap();
    }

    #[test]
    fn get_executed_commands_records_every_attempt_in_order() {
        let system = SystemMock::new().add_command_output("echo one", b"");

        let mut first = SystemCommand::new("echo");
        first.arg("one");
        system.run_command(&first).unwrap();

        let mut second = SystemCommand::new("echo");
        second.arg("two");
        system.run_command(&second).ok();

        // The failed attempt is recorded too, since the host was still asked
        // to run it.
        assert_eq!(system.get_executed_commands(), vec!["echo one", "echo two"]);
    }
}
