use crate::error::{Error, Result};
use crate::platform::{Network, ReadWrite, SystemMock};
use std::io::{Cursor, Read, Write};
use std::time::Duration;

// Where a bind starts looking. A bind skips every port the host already knows
// about, so seeding one is always safe, but a test that names a port without
// seeding it is not protected. The base sits high to keep that out of the
// range tests reach for by hand.
const FIRST_FREE_PORT: u16 = 60000;

// What a listening port sends the moment a connection lands. Only the presence
// of these bytes carries meaning, never their content.
const GREETING: &[u8] = b"SSH-2.0-mock\r\n";

// What a port does when it is dialled. A port the host has never heard of is
// one nothing listens on and nothing has claimed, so it is free.
#[derive(Clone, Copy)]
enum PortState {
    // Accepts and greets the caller, the way a ready service does.
    Listening,
    // Accepts and then says nothing, standing in for a forward that is bound
    // before the service behind it can answer.
    Silent,
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
}

impl NetworkMock {
    fn add(&mut self, port: u16, state: PortState) {
        self.ports.retain(|(known, _)| *known != port);
        self.ports.push((port, state));
    }

    fn get_connected(&self) -> Vec<u16> {
        self.connected.clone()
    }

    fn connect(&mut self, port: u16) -> Result<Box<dyn ReadWrite>> {
        self.connected.push(port);

        match self.find(port) {
            Some(PortState::Listening) => Ok(Box::new(StreamMock::new(GREETING))),
            Some(PortState::Silent) => Ok(Box::new(StreamMock::new(b""))),
            _ => Err(Error::Io(std::io::ErrorKind::ConnectionRefused.into())),
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

// A connection to a seeded listener. Reading past the greeting reports a
// timeout rather than end of file, because a real socket held open by a peer
// that has stopped talking blocks until its read timeout expires. A reader
// that treats end of file as success would see a silent port as a talking one.
struct StreamMock {
    greeting: Cursor<Vec<u8>>,
}

impl StreamMock {
    fn new(greeting: &[u8]) -> Self {
        Self {
            greeting: Cursor::new(greeting.to_vec()),
        }
    }
}

impl Read for StreamMock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.greeting.read(buf)? {
            0 if !buf.is_empty() => Err(std::io::ErrorKind::TimedOut.into()),
            count => Ok(count),
        }
    }
}

impl Write for StreamMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl SystemMock {
    // A port that accepts and then greets the caller, the way a ready sshd
    // does.
    pub fn add_open_port(self, port: u16) -> Self {
        self.add_port_state(port, PortState::Listening)
    }

    // A port that accepts and then says nothing, the way QEMU hostfwd does
    // while the guest is still booting.
    pub fn add_silent_port(self, port: u16) -> Self {
        self.add_port_state(port, PortState::Silent)
    }

    fn add_port_state(self, port: u16, state: PortState) -> Self {
        self.network.borrow_mut().add(port, state);
        self
    }

    // Every port the host was asked to connect to, seeded or not, in order.
    pub fn get_connected_ports(&self) -> Vec<u16> {
        self.network.borrow().get_connected()
    }
}

impl Network for SystemMock {
    // The timeout has no meaning here, since a seeded listener answers at once
    // and an unseeded one refuses at once.
    fn connect_port(&self, port: u16, _timeout: Duration) -> Result<Box<dyn ReadWrite>> {
        self.network.borrow_mut().connect(port)
    }

    fn bind_port(&self) -> Result<u16> {
        self.network.borrow_mut().bind()
    }
}

#[cfg(test)]
mod tests {
    use super::FIRST_FREE_PORT;
    use crate::error::Error;
    use crate::platform::{Network, SystemMock};
    use std::io::Read;
    use std::time::Duration;

    #[test]
    fn connect_port_reads_something_back_from_an_open_port() {
        let system = SystemMock::new().add_open_port(22);

        let mut stream = system.connect_port(22, Duration::from_secs(1)).unwrap();

        // Only the presence of the bytes matters, since that is what tells a
        // service that can answer from one that cannot yet.
        assert!(stream.read(&mut [0]).unwrap() > 0);
    }

    #[test]
    fn connect_port_times_out_reading_a_silent_port() {
        let system = SystemMock::new().add_silent_port(22);

        let mut stream = system.connect_port(22, Duration::from_secs(1)).unwrap();

        // A peer that has sent nothing has to look like a stalled read rather
        // than an ended one, otherwise a caller that treats end of file as
        // success reads a silent port as a talking one.
        assert_eq!(
            stream.read(&mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::TimedOut
        );
    }

    #[test]
    fn connect_port_is_refused_when_nothing_listens() {
        let system = SystemMock::new();

        assert!(matches!(
            system.connect_port(22, Duration::from_secs(1)),
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused
        ));
    }

    #[test]
    fn get_connected_ports_records_every_attempt_in_order() {
        let system = SystemMock::new().add_open_port(22);

        system.connect_port(22, Duration::from_secs(1)).unwrap();
        system.connect_port(80, Duration::from_secs(1)).ok();

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

    #[test]
    fn connect_is_refused_on_a_port_a_bind_handed_out() {
        let system = SystemMock::new();

        // A bind only reserves the number, so nothing answers on it until
        // whoever asked for it starts listening.
        let port = system.bind_port().unwrap();

        assert!(system.connect_port(port, Duration::from_secs(1)).is_err());
    }
}
