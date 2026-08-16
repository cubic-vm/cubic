use crate::error::{Error, Result};
use crate::platform::{Network, OsSystem, ReadWrite};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

impl Network for OsSystem {
    fn connect_port(&self, port: u16, timeout: Duration) -> Result<Box<dyn ReadWrite>> {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        Ok(Box::new(stream))
    }

    // The listener is dropped right away, so the port is only reserved for as
    // long as it takes the caller to hand it to whoever binds it for real.
    fn bind_port(&self) -> Result<u16> {
        TcpListener::bind("127.0.0.1:0")
            .map_err(|_| Error::NoPortAvailable)?
            .local_addr()
            .map(|addr| addr.port())
            .map_err(|_| Error::NoPortAvailable)
    }
}
