use crate::error::Result;
use crate::platform::Socket;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

pub trait Network {
    // Opens a loopback connection to `port`. The timeout bounds reads on
    // the returned stream, not the connect itself.
    fn connect_port(&self, port: u16, timeout: Duration) -> Result<Box<dyn Read>>;
    // Opens a connection to the Unix domain socket at `path`.
    fn connect_socket(&self, path: &Path, timeout: Option<Duration>) -> Result<Box<dyn Socket>>;
    // Takes a free loopback port from the host and reports its number.
    fn bind_port(&self) -> Result<u16>;
}
