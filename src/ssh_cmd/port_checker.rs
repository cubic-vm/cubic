use std::net::TcpStream;
use std::time::Duration;

pub struct PortChecker {
    pub port: u16,
}

impl PortChecker {
    pub fn new(port: u16) -> Self {
        PortChecker { port }
    }

    pub fn try_connect(&self) -> bool {
        let mut buf = [0];
        TcpStream::connect(format!("127.0.0.1:{}", &self.port))
            .and_then(|stream| {
                stream.set_read_timeout(Some(Duration::from_secs(1)))?;
                stream.peek(&mut buf)
            })
            .is_ok()
    }
}
