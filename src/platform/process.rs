use crate::error::Result;
use std::process::Command;
use std::time::Duration;
use tokio::io::AsyncRead;

pub type ChildOutput = Box<dyn AsyncRead + Unpin>;

pub trait Child {
    fn try_wait(&mut self) -> Option<bool>;
    fn take_stdout(&mut self) -> Option<ChildOutput>;
    fn take_stderr(&mut self) -> Option<ChildOutput>;
}

pub trait Process {
    fn run_and_wait(
        &self,
        command: Command,
        stdin: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Vec<u8>>;
    fn run_detached(&self, command: Command) -> Result<Box<dyn Child>>;
}
