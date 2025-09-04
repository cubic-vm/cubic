use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub struct PortChecker;

impl PortChecker {
    pub fn new() -> Self {
        PortChecker {}
    }

    pub fn is_open(&self, port: u16) -> bool {
        let mut buf = [0];
        TcpStream::connect(format!("127.0.0.1:{port}"))
            .and_then(|stream| {
                stream.set_read_timeout(Some(Duration::from_secs(1)))?;
                stream.peek(&mut buf)
            })
            .is_ok()
    }

    pub fn get_new_port(&self) -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }
}
