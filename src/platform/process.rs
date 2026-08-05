use crate::error::Result;
use crate::util::SystemCommand;

pub trait Process {
    fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>>;
    fn spawn_command(&self, command: &SystemCommand) -> Result<()>;

    fn exists_process(&self, pid: u64) -> bool;
    fn kill_process(&self, pid: u64) -> Result<()>;
}
