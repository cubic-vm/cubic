use crate::error::{Error, Result};
use crate::platform::{Network, OsSystem, Socket};
use socket2::{Domain, SockAddr, Type};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

impl Network for OsSystem {
    fn connect_port(&self, port: u16, timeout: Duration) -> Result<Box<dyn Read>> {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        Ok(Box::new(stream))
    }

    fn connect_socket(&self, path: &Path, timeout: Option<Duration>) -> Result<Box<dyn Socket>> {
        let socket = socket2::Socket::new(Domain::UNIX, Type::STREAM, None)?;
        socket.connect(&SockAddr::unix(path)?)?;
        socket.set_read_timeout(timeout)?;
        socket.set_write_timeout(timeout)?;
        Ok(Box::new(socket))
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

impl Socket for socket2::Socket {
    fn try_clone(&self) -> Result<Box<dyn Socket>> {
        Ok(Box::new(socket2::Socket::try_clone(self)?))
    }
}
