use crate::platform::Network;
use std::time::Duration;
use tokio::io::AsyncReadExt;

// QEMU accepts connections before the guest is up, which then stay silent.
const SSH_TIMEOUT: Duration = Duration::from_secs(2);

pub struct PortChecker;

impl PortChecker {
    pub fn new() -> Self {
        PortChecker {}
    }

    // Whether something listens on a port.
    pub async fn is_open(&self, system: &dyn Network, port: u16) -> bool {
        system.connect_stream(port).await.is_ok()
    }

    // Every SSH server opens with an `SSH-` line, so no key exchange is needed.
    pub async fn is_ssh_ready(&self, system: &dyn Network, port: u16) -> bool {
        let banner = async {
            let Ok(mut stream) = system.connect_stream(port).await else {
                return false;
            };
            let mut prefix = [0u8; 4];
            stream.read_exact(&mut prefix).await.is_ok() && &prefix == b"SSH-"
        };
        tokio::time::timeout(SSH_TIMEOUT, banner)
            .await
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[tokio::test]
    async fn test_is_open_true_for_a_port_that_listens() {
        let system = SystemMock::new().add_open_port(22);

        assert!(PortChecker::new().is_open(&system, 22).await);
        assert_eq!(system.get_connected_ports(), vec![22]);
    }

    #[tokio::test]
    async fn test_is_open_false_when_nothing_listens() {
        let system = SystemMock::new();

        assert!(!PortChecker::new().is_open(&system, 22).await);
    }

    #[tokio::test]
    async fn test_is_ssh_ready_only_for_an_ssh_banner() {
        let checker = PortChecker::new();
        let system = SystemMock::new()
            .add_replying_port(22, b"SSH-2.0-OpenSSH_10.3\r\n")
            .add_replying_port(80, b"HTTP/1.1 400\r\n")
            .add_open_port(23);

        assert!(checker.is_ssh_ready(&system, 22).await);
        assert!(!checker.is_ssh_ready(&system, 80).await);
        assert!(!checker.is_ssh_ready(&system, 23).await);
        assert!(!checker.is_ssh_ready(&system, 24).await);
    }
}
