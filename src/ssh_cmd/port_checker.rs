use std::net::TcpListener;

pub struct PortChecker;

impl PortChecker {
    pub fn new() -> Self {
        PortChecker {}
    }

    pub fn is_open(&self, port: u16) -> bool {
        TcpListener::bind(format!("127.0.0.1:{port}")).is_err()
    }

    pub fn get_new_port(&self) -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }
}
