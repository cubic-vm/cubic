use crate::error::Result;
use crate::util::SystemCommand;
use std::time::Duration;

pub trait Process {
    fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>>;
    // Runs a command that is not expected to end by itself and waits for a
    // marker on its stdout. The marker means the command came up, so it is
    // killed right away. A command that ends first, or one that never prints
    // the marker before the deadline, reports a failure carrying its stderr.
    fn run_command_until_output(
        &self,
        command: &SystemCommand,
        marker: &str,
        timeout: Duration,
    ) -> Result<()>;
    fn spawn_command(&self, command: &SystemCommand) -> Result<()>;

    fn exists_process(&self, pid: u64) -> bool;
    fn kill_process(&self, pid: u64) -> Result<()>;
}
