use crate::error::Result;
use std::pin::Pin;
use std::process::Command;
use tokio::io::{AsyncRead, AsyncWrite};

pub type ChildInput = Box<dyn AsyncWrite + Unpin>;
pub type ChildOutput = Box<dyn AsyncRead + Unpin>;
pub type ChildExit<'a> = Pin<Box<dyn Future<Output = bool> + 'a>>;

pub trait Child {
    fn wait(&mut self) -> ChildExit<'_>;
    fn take_stdin(&mut self) -> ChildInput;
    fn take_stdout(&mut self) -> ChildOutput;
    fn take_stderr(&mut self) -> ChildOutput;
}

pub trait Process {
    // A detached child outlives cubic. Any other child dies with its handle.
    fn spawn(&self, command: Command, detach: bool) -> Result<Box<dyn Child>>;
}
