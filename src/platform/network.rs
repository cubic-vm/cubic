use crate::error::Result;
use crate::platform::ReadWrite;
use std::time::Duration;

pub trait Network {
    // Opens a loopback connection to `port`. The timeout bounds reads and
    // writes on the returned stream, not the connect itself.
    fn connect_port(&self, port: u16, timeout: Duration) -> Result<Box<dyn ReadWrite>>;
    // Takes a free loopback port from the host and reports its number.
    fn bind_port(&self) -> Result<u16>;
}
