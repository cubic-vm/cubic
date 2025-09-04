use std::net::TcpListener;

pub fn generate_random_ssh_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    addr.port()
}
