use crate::platform::System;

pub struct PortChecker;

impl PortChecker {
    pub fn new() -> Self {
        PortChecker {}
    }

    // Whether something listens on a port.
    pub async fn is_open(&self, system: &dyn System, port: u16) -> bool {
        system.connect_stream(port).await.is_ok()
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
}
