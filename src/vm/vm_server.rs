use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::{Listener, Socket, System};
use crate::qemu::QemuMonitorClient;
use crate::vm::VmControl;
use crate::vm::vm_message::{INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR, VmMessage};
use std::convert::Infallible;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

const ACCEPT_INTERVAL: Duration = Duration::from_millis(50);
const SERVER_ERROR: i64 = -32000;

pub struct VmServer<'a> {
    system: &'a dyn System,
    path: PathBuf,
    listener: Box<dyn Listener>,
}

impl<'a> VmServer<'a> {
    pub fn bind(system: &'a dyn System, env: &Environment, instance: &Instance) -> Result<Self> {
        let path = env.get_api_socket(&instance.name);
        if system.connect_socket(&path, None).is_ok() {
            return Err(Error::InstanceNotStopped(instance.name.clone()));
        }
        system.remove_file(&path).ok();
        let listener = system.bind_socket(&path)?;
        Ok(VmServer {
            system,
            path,
            listener,
        })
    }

    // A fresh monitor per use keeps the single QMP client slot free
    pub async fn run(&self, build_monitor: impl Fn() -> QemuMonitorClient) -> Infallible {
        loop {
            match self.accept_client() {
                Some(socket) => self.handle_client(socket, &mut build_monitor()),
                None => tokio::time::sleep(ACCEPT_INTERVAL).await,
            }
        }
    }

    fn accept_client(&self) -> Option<Box<dyn Socket>> {
        self.listener.accept().ok()
    }

    fn handle_client(&self, socket: Box<dyn Socket>, monitor: &mut QemuMonitorClient) {
        let mut client = BufReader::new(socket);
        let mut line = String::new();
        // A probe connects and closes without sending anything
        if !matches!(client.read_line(&mut line), Ok(count) if count > 0) {
            return;
        }
        let response = match serde_json::from_str(&line) {
            Ok(VmMessage::Request { method, .. }) => Self::answer_request(&method, monitor),
            Ok(_) => VmMessage::build_error(INVALID_REQUEST, "Invalid Request"),
            Err(_) => VmMessage::build_error(PARSE_ERROR, "Parse error"),
        };
        if let Ok(mut reply) = serde_json::to_string(&response) {
            reply.push('\n');
            client.get_mut().write_all(reply.as_bytes()).ok();
        }
    }

    fn answer_request(method: &str, monitor: &mut QemuMonitorClient) -> VmMessage {
        if method != "stop" {
            return VmMessage::build_error(METHOD_NOT_FOUND, "Method not found");
        }
        match monitor.stop() {
            Ok(()) => VmMessage::build_result(),
            Err(e) => VmMessage::build_error(SERVER_ERROR, &e.to_string()),
        }
    }
}

impl Drop for VmServer<'_> {
    fn drop(&mut self) {
        self.system.remove_file(&self.path).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::QMP_REPLIES;
    use crate::platform::{FileSystem, Network, SystemMock};
    use crate::vm::VmClient;
    use std::path::Path;
    use std::sync::Arc;

    const CLIENT_SOCKET: &str = "/client.sock";

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_bind_fails_while_a_vm_answers() {
        let env = Environment::default();
        let system = SystemMock::new().add_socket(env.get_api_socket("test"), b"");

        let result = VmServer::bind(&system, &env, &build_instance());

        assert!(matches!(result, Err(Error::InstanceNotStopped(name)) if name == "test"));
    }

    #[test]
    fn test_bind_replaces_a_leftover_socket_file() {
        let env = Environment::default();
        let path = env.get_api_socket("test");
        let system = SystemMock::new().add_file(&path, b"");

        let _server = VmServer::bind(&system, &env, &build_instance()).unwrap();

        assert!(!system.exists_path(&path));
        assert!(VmClient::connect(&system, &env, &build_instance()).is_ok());
    }

    fn send_line(line: &str) -> (String, String) {
        let env = Environment::default();
        let monitor_path = env.get_monitor_socket("test");
        let system = SystemMock::new()
            .add_socket(&monitor_path, QMP_REPLIES)
            .add_socket(CLIENT_SOCKET, line.as_bytes());
        let system = Arc::new(system);
        let server = VmServer::bind(system.as_ref(), &env, &build_instance()).unwrap();
        let socket = system.connect_socket(Path::new(CLIENT_SOCKET), None);
        let mut monitor = QemuMonitorClient::new(system.clone(), &env, &build_instance());

        server.handle_client(socket.unwrap(), &mut monitor);

        let reply = system.get_socket_output(CLIENT_SOCKET);
        (reply, system.get_socket_output(monitor_path))
    }

    #[test]
    fn test_stop_powers_the_guest_down_and_replies() {
        let (reply, qmp) = send_line("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"stop\"}\n");

        assert_eq!(reply, "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n");
        assert!(qmp.ends_with(r#"{"id":"1","execute":"system_powerdown"}"#));
    }

    #[test]
    fn test_unknown_method_is_rejected_without_touching_qemu() {
        let (reply, qmp) = send_line("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"jump\"}\n");

        assert!(reply.contains(r#""error":{"code":-32601"#));
        assert!(qmp.is_empty());
    }

    #[test]
    fn test_malformed_line_gets_a_parse_error() {
        let (reply, _) = send_line("not json\n");

        assert!(reply.contains(r#""error":{"code":-32700"#));
    }
}
