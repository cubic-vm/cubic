use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::{Listener, Socket, System};
use crate::qemu::QemuMonitorClient;
use crate::ssh::PortChecker;
use crate::vm::vm_message::{INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR, VmMessage, VmMethod};
use crate::vm::{VmControl, VmState};
use serde_json::Value;
use std::cell::Cell;
use std::convert::Infallible;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

const ACCEPT_INTERVAL: Duration = Duration::from_millis(50);
const SERVER_ERROR: i64 = -32000;
const SSH_PROBE_TIMEOUT: Duration = Duration::from_millis(200);

pub struct VmServer<'a> {
    system: &'a dyn System,
    path: PathBuf,
    ssh_port: u16,
    listener: Box<dyn Listener>,
    state: Cell<VmState>,
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
            ssh_port: instance.ssh_port,
            listener,
            state: Cell::new(VmState::Starting),
        })
    }

    // A fresh monitor per use keeps the single QMP client slot free
    pub async fn run(&self, build_monitor: impl Fn() -> QemuMonitorClient) -> Infallible {
        loop {
            match self.accept_client() {
                Some(socket) => self.handle_client(socket, &mut build_monitor()).await,
                None => tokio::time::sleep(ACCEPT_INTERVAL).await,
            }
        }
    }

    fn accept_client(&self) -> Option<Box<dyn Socket>> {
        self.listener.accept().ok()
    }

    async fn handle_client(&self, socket: Box<dyn Socket>, monitor: &mut QemuMonitorClient) {
        let mut client = BufReader::new(socket);
        let mut line = String::new();
        if !matches!(client.read_line(&mut line), Ok(count) if count > 0) {
            return;
        }
        let response = match serde_json::from_str(&line) {
            Ok(VmMessage::Request { method, .. }) => self.answer_request(method, monitor).await,
            Ok(_) => VmMessage::build_error(INVALID_REQUEST, "Invalid Request"),
            Err(_) => VmMessage::build_error(PARSE_ERROR, "Parse error"),
        };
        if let Ok(mut reply) = serde_json::to_string(&response) {
            reply.push('\n');
            client.get_mut().write_all(reply.as_bytes()).ok();
        }
    }

    async fn answer_request(&self, method: VmMethod, monitor: &mut QemuMonitorClient) -> VmMessage {
        let result = match method {
            VmMethod::Stop => self.stop(monitor, false),
            VmMethod::Quit => self.stop(monitor, true),
            VmMethod::GetState => serde_json::to_value(self.get_state().await).map_err(Into::into),
            VmMethod::Unknown => {
                return VmMessage::build_error(METHOD_NOT_FOUND, "Method not found");
            }
        };
        match result {
            Ok(value) => VmMessage::build_result(value),
            Err(e) => VmMessage::build_error(SERVER_ERROR, &e.to_string()),
        }
    }

    fn stop(&self, monitor: &mut QemuMonitorClient, kill: bool) -> Result<Value> {
        if kill {
            monitor.quit()?;
        } else {
            monitor.stop()?;
        }
        self.state.set(VmState::Stopping);
        Ok(Value::Null)
    }

    async fn get_state(&self) -> VmState {
        if self.state.get() == VmState::Starting && self.is_ssh_ready().await {
            self.state.set(VmState::Running);
        }
        self.state.get()
    }

    async fn is_ssh_ready(&self) -> bool {
        let checker = PortChecker::new();
        let probe = checker.is_ssh_ready(self.system, self.ssh_port);
        let ready = tokio::time::timeout(SSH_PROBE_TIMEOUT, probe).await;
        ready.unwrap_or(false)
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

    async fn ask(server: &VmServer<'_>, monitor: &mut QemuMonitorClient, line: &str) -> String {
        let system = SystemMock::new().add_socket(CLIENT_SOCKET, line.as_bytes());
        let path = Path::new(CLIENT_SOCKET);
        let socket = system.connect_socket(path, None).unwrap();
        server.handle_client(socket, monitor).await;
        system.get_socket_output(CLIENT_SOCKET)
    }

    fn build_monitor(system: SystemMock) -> (Arc<SystemMock>, QemuMonitorClient) {
        let system = Arc::new(system);
        let env = Environment::default();
        let monitor = QemuMonitorClient::new(system.clone(), &env, &build_instance());
        (system, monitor)
    }

    fn build_running_monitor() -> (Arc<SystemMock>, QemuMonitorClient) {
        let path = Environment::default().get_monitor_socket("test");
        build_monitor(SystemMock::new().add_socket(path, QMP_REPLIES))
    }

    async fn send_line(line: &str) -> (String, String) {
        let system = SystemMock::new();
        let server = VmServer::bind(&system, &Environment::default(), &build_instance()).unwrap();
        let (qemu, mut monitor) = build_running_monitor();

        let reply = ask(&server, &mut monitor, line).await;

        let qmp = qemu.get_socket_output(Environment::default().get_monitor_socket("test"));
        (reply, qmp)
    }

    fn build_request(method: &str) -> String {
        format!("{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{method}\"}}\n")
    }

    #[tokio::test]
    async fn test_stop_and_quit_reach_qemu_and_reply() {
        for (method, command) in [("stop", "system_powerdown"), ("quit", "quit")] {
            let (reply, qmp) = send_line(&build_request(method)).await;

            assert_eq!(reply, "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":null}\n");
            assert!(qmp.ends_with(&format!(r#"{{"id":"1","execute":"{command}"}}"#)));
        }
    }

    #[tokio::test]
    async fn test_unknown_method_is_rejected_without_touching_qemu() {
        let (reply, qmp) = send_line(&build_request("jump")).await;

        assert!(reply.contains(r#""error":{"code":-32601"#));
        assert!(qmp.is_empty());
    }

    #[tokio::test]
    async fn test_malformed_line_gets_a_parse_error() {
        let (reply, _) = send_line("not json\n").await;

        assert!(reply.contains(r#""error":{"code":-32700"#));
    }

    #[tokio::test]
    async fn test_get_state_follows_the_lifecycle() {
        let system = SystemMock::new();
        let instance = build_instance();
        let server = VmServer::bind(&system, &Environment::default(), &instance).unwrap();
        let get_state = build_request("get-state");
        let (_, mut monitor) = build_running_monitor();

        let reply = ask(&server, &mut monitor, &get_state).await;
        assert!(reply.contains(r#""result":"starting""#));

        system.insert_replying_port(instance.ssh_port, b"SSH-2.0-OpenSSH_10.3\r\n");
        let reply = ask(&server, &mut monitor, &get_state).await;
        assert!(reply.contains(r#""result":"running""#));

        system.insert_replying_port(instance.ssh_port, b"");
        let reply = ask(&server, &mut monitor, &get_state).await;
        assert!(reply.contains(r#""result":"running""#));

        ask(&server, &mut monitor, &build_request("stop")).await;
        let reply = ask(&server, &mut monitor, &get_state).await;
        assert!(reply.contains(r#""result":"stopping""#));
    }
}
