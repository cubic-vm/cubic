use crate::error::Result;
use std::process::Command;
use std::time::Duration;

pub trait Process {
    fn run_and_wait(
        &self,
        command: Command,
        stdin: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>>;
    fn run_detached(&self, command: Command) -> Result<()>;

    fn exists_process(&self, pid: u64) -> bool;
}
