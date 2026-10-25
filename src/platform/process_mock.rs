use crate::error::{Error, Result};
use crate::platform::{Process, SystemMock};
use crate::util::CommandLine;
use std::collections::{HashMap, HashSet};
use std::process::Command;
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

    fn run(&mut self, command: &Command) -> Result<Vec<u8>> {
        let line = command.get_command();
        self.executed.push(line.clone());

        match self.results.get(line.as_str()) {
            None => Err(Error::SystemCommandNotFound(
                command.get_program().to_string_lossy().into_owned(),
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
    fn run_and_wait(
        &self,
        command: Command,
        _stdin: Option<&str>,
        _timeout: Option<Duration>,
    ) -> Result<Vec<u8>> {
        self.commands.lock().unwrap().run(&command)
    }

    // A detached start has nothing to wait for, so it only reports whether the
    // host could launch the command at all.
    fn run_detached(&self, command: Command) -> Result<()> {
        self.commands.lock().unwrap().run(&command).map(|_| ())
    }

    fn exists_process(&self, pid: u64) -> bool {
        self.processes.lock().unwrap().exists(pid)
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::platform::{Process, SystemMock};
    use std::process::Command;

    #[test]
    fn exists_process_only_finds_seeded_pids() {
        let system = SystemMock::new().add_process(42);

        assert!(system.exists_process(42));
        assert!(!system.exists_process(43));
    }

    #[test]
    fn run_and_wait_returns_the_seeded_output() {
        let system = SystemMock::new().add_command_output("echo hello", b"hello\n");

        let mut command = Command::new("echo");
        command.arg("hello");

        assert_eq!(
            system.run_and_wait(command, None, None).unwrap(),
            b"hello\n"
        );
    }

    #[test]
    fn run_and_wait_fails_for_an_unseeded_command() {
        let system = SystemMock::new();

        assert!(matches!(
            system.run_and_wait(Command::new("qemu-img"), None, None),
            Err(Error::SystemCommandNotFound(program)) if program == "qemu-img"
        ));
    }

    #[test]
    fn run_and_wait_reports_a_seeded_failure_with_its_stderr() {
        let system = SystemMock::new().add_failing_command("qemu-img resize", "no such file");

        let mut command = Command::new("qemu-img");
        command.arg("resize");

        assert!(matches!(
            system.run_and_wait(command, None, None),
            Err(Error::SystemCommandFailed(line, stderr))
                if line == "qemu-img resize" && stderr == "no such file"
        ));
    }

    #[test]
    fn run_and_wait_matches_on_the_arguments() {
        let system = SystemMock::new().add_command_output("qemu-img info a", b"a");

        let mut other = Command::new("qemu-img");
        other.arg("info").arg("b");

        assert!(system.run_and_wait(other, None, None).is_err());
    }

    #[test]
    fn run_detached_succeeds_without_returning_output() {
        let system = SystemMock::new().add_command_output("qemu-system-x86_64", b"ignored");

        system
            .run_detached(Command::new("qemu-system-x86_64"))
            .unwrap();
    }

    #[test]
    fn get_executed_commands_records_every_attempt_in_order() {
        let system = SystemMock::new().add_command_output("echo one", b"");

        let mut first = Command::new("echo");
        first.arg("one");
        system.run_and_wait(first, None, None).unwrap();

        let mut second = Command::new("echo");
        second.arg("two");
        system.run_and_wait(second, None, None).ok();

        // The failed attempt is recorded too, since the host was still asked
        // to run it.
        assert_eq!(system.get_executed_commands(), vec!["echo one", "echo two"]);
    }
}
