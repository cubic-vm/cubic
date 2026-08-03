use crate::error::Error;
use crate::platform::System;
use std::io::Read;
use std::time::Duration;

const PEEK_TIMEOUT_SECS: u64 = 1;

pub struct PortChecker;

impl PortChecker {
    pub fn new() -> Self {
        PortChecker {}
    }

    // Whether a port both accepts a connection and greets whoever connects.
    // Accepting alone is too weak a signal, because QEMU hostfwd binds the
    // host port as soon as the network device comes up, long before the guest
    // service behind it can answer.
    pub fn is_open(&self, system: &dyn System, port: u16) -> bool {
        let mut buf = [0];
        system
            .connect_port(port, Duration::from_secs(PEEK_TIMEOUT_SECS))
            .and_then(|mut stream| stream.read(&mut buf).map_err(Error::from))
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_is_open_true_for_a_port_that_greets() {
        let system = SystemMock::new().add_open_port(22);

        assert!(PortChecker::new().is_open(&system, 22));
        assert_eq!(system.get_connected_ports(), vec![22]);
    }

    #[test]
    fn test_is_open_false_for_a_port_that_stays_silent() {
        // QEMU hostfwd accepts on the host port as soon as the network device
        // comes up, so a port that accepts and then says nothing is a guest
        // that cannot answer yet.
        let system = SystemMock::new().add_silent_port(22);

        assert!(!PortChecker::new().is_open(&system, 22));
    }

    #[test]
    fn test_is_open_false_when_nothing_listens() {
        let system = SystemMock::new();

        assert!(!PortChecker::new().is_open(&system, 22));
    }
}
