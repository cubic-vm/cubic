use crate::error::{Error, Result};
use crate::platform::{AsyncSocket, AsyncSocketFuture, Network, Socket, SystemMock};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Where a bind starts looking. A bind skips every port the host already knows
// about, so seeding one is always safe, but a test that names a port without
// seeding it is not protected. The base sits high to keep that out of the
// range tests reach for by hand.
const FIRST_FREE_PORT: u16 = 60000;

// What a port does when it is dialled. A port the host has never heard of is
// one nothing listens on and nothing has claimed, so it is free.
#[derive(Clone, Copy)]
enum PortState {
    // Accepts the caller, the way a running service does.
    Listening,
    // Handed out by a bind. Nothing listens on it, since a bind only reserves
    // the number and leaves the listening to whoever asked.
    Bound,
}

// Every port the host knows about, in the order it learned of them, alongside
// the record of which ones were dialled. A port that is absent is one nothing
// listens on, so connecting to it is refused rather than quietly succeeding.
#[derive(Default)]
pub struct NetworkMock {
    ports: Vec<(u16, PortState)>,
    connected: Vec<u16>,
    sockets: Vec<(PathBuf, SocketMock)>,
}

impl NetworkMock {
    fn find_socket(&self, path: &Path) -> Option<&SocketMock> {
        self.sockets
            .iter()
            .find(|(known, _)| known == path)
            .map(|(_, socket)| socket)
    }

    fn add(&mut self, port: u16, state: PortState) {
        self.ports.retain(|(known, _)| *known != port);
        self.ports.push((port, state));
    }

    fn get_connected(&self) -> Vec<u16> {
        self.connected.clone()
    }

    fn connect(&mut self, port: u16) -> Result<Box<dyn AsyncSocket>> {
        self.connected.push(port);

        match self.find(port) {
            Some(PortState::Listening) => Ok(Box::new(tokio::io::duplex(1).0)),
            _ => Err(Error::ConnectionFailed(
                port,
                std::io::ErrorKind::ConnectionRefused.into(),
            )),
        }
    }

    // Takes the lowest port nothing has claimed yet and records the claim, so
    // a second bind cannot hand out the same number.
    fn bind(&mut self) -> Result<u16> {
        let port = (FIRST_FREE_PORT..=u16::MAX)
            .find(|port| self.find(*port).is_none())
            .ok_or(Error::NoPortAvailable)?;
        self.add(port, PortState::Bound);
        Ok(port)
    }

    fn find(&self, port: u16) -> Option<PortState> {
        self.ports
            .iter()
            .find(|(known, _)| *known == port)
            .map(|(_, state)| *state)
    }
}

// A connection to a seeded socket. Reading past the replies reports a
// timeout rather than end of file, because a real socket held open by a peer
// that has stopped talking blocks until its read timeout expires.
#[derive(Clone)]
struct SocketMock {
    replies: Cursor<Vec<u8>>,
    written: Arc<Mutex<Vec<u8>>>,
}

impl SocketMock {
    fn new(replies: &[u8]) -> Self {
        Self {
            replies: Cursor::new(replies.to_vec()),
            written: Arc::default(),
        }
    }
}

impl Read for SocketMock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.replies.read(buf)? {
            0 if !buf.is_empty() => Err(std::io::ErrorKind::TimedOut.into()),
            count => Ok(count),
        }
    }
}

impl Write for SocketMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.written.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Socket for SocketMock {
    fn try_clone(&self) -> Result<Box<dyn Socket>> {
        Ok(Box::new(self.clone()))
    }
}

impl SystemMock {
    pub fn add_socket(self, path: &str, replies: &[u8]) -> Self {
        self.network
            .lock()
            .unwrap()
            .sockets
            .push((PathBuf::from(path), SocketMock::new(replies)));
        self
    }

    pub fn get_socket_output(&self, path: &str) -> String {
        self.network
            .lock()
            .unwrap()
            .find_socket(Path::new(path))
            .map(|socket| String::from_utf8_lossy(&socket.written.lock().unwrap()).into_owned())
            .unwrap_or_default()
    }

    // A port that accepts the caller, the way a running service does.
    pub fn add_open_port(self, port: u16) -> Self {
        self.network.lock().unwrap().add(port, PortState::Listening);
        self
    }

    // Every port the host was asked to connect to, seeded or not, in order.
    pub fn get_connected_ports(&self) -> Vec<u16> {
        self.network.lock().unwrap().get_connected()
    }
}

impl Network for SystemMock {
    fn connect_stream(&self, port: u16) -> AsyncSocketFuture<'_> {
        Box::pin(std::future::ready(
            self.network.lock().unwrap().connect(port),
        ))
    }

    fn connect_socket(&self, path: &Path, _timeout: Option<Duration>) -> Result<Box<dyn Socket>> {
        self.network
            .lock()
            .unwrap()
            .find_socket(path)
            .map(|socket| Box::new(socket.clone()) as Box<dyn Socket>)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::ConnectionRefused).into())
    }

    fn bind_port(&self) -> Result<u16> {
        self.network.lock().unwrap().bind()
    }
}

#[cfg(test)]
mod tests {
    use super::FIRST_FREE_PORT;
    use crate::error::Error;
    use crate::platform::{Network, SystemMock};

    #[tokio::test]
    async fn connect_stream_is_refused_when_nothing_listens() {
        let system = SystemMock::new();

        assert!(matches!(
            system.connect_stream(22).await,
            Err(Error::ConnectionFailed(22, e)) if e.kind() == std::io::ErrorKind::ConnectionRefused
        ));
    }

    #[tokio::test]
    async fn get_connected_ports_records_every_attempt_in_order() {
        let system = SystemMock::new().add_open_port(22);

        assert!(system.connect_stream(22).await.is_ok());
        system.connect_stream(80).await.ok();

        assert_eq!(system.get_connected_ports(), vec![22, 80]);
    }

    #[test]
    fn bind_port_claims_successive_free_ports() {
        let system = SystemMock::new();

        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT);
        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT + 1);
    }

    #[test]
    fn bind_port_skips_a_port_the_test_already_claimed() {
        let system = SystemMock::new().add_open_port(FIRST_FREE_PORT);

        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT + 1);
    }

    #[tokio::test]
    async fn connect_is_refused_on_a_port_a_bind_handed_out() {
        let system = SystemMock::new();

        // A bind only reserves the number, so nothing answers on it until
        // whoever asked for it starts listening.
        let port = system.bind_port().unwrap();

        assert!(system.connect_stream(port).await.is_err());
    }
}
