use crate::error::Result;
use crate::platform::{AsyncSocketFuture, Socket};
use std::path::Path;
use std::time::Duration;

pub trait Network {
    // Opens a loopback connection to `port`.
    fn connect_stream(&self, port: u16) -> AsyncSocketFuture<'_>;
    // Opens a connection to the Unix domain socket at `path`.
    fn connect_socket(&self, path: &Path, timeout: Option<Duration>) -> Result<Box<dyn Socket>>;
    // Takes a free loopback port from the host and reports its number.
    fn bind_port(&self) -> Result<u16>;
}
