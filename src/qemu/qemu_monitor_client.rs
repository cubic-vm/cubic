use crate::error::{Error, Result};
use crate::models::{Environment, Instance, PortForward};
use crate::platform::{Socket, System};
use crate::qemu::{NETDEV_ID, QmpMessage};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Leaves room for a running check queued ahead on the single client slot
const QMP_TIMEOUT: Duration = Duration::from_secs(1);

pub struct QemuMonitorClient {
    system: Arc<dyn System>,
    socket: PathBuf,
    name: String,
    counter: u64,
    stream: Option<BufReader<Box<dyn Socket>>>,
}

impl QemuMonitorClient {
    pub fn new(system: Arc<dyn System>, env: &Environment, instance: &Instance) -> Self {
        QemuMonitorClient {
            system,
            socket: env.get_monitor_socket(&instance.name),
            name: instance.name.clone(),
            counter: 0,
            stream: None,
        }
    }

    // A plain connect without the handshake keeps the QMP client slot free
    pub fn is_running(&self) -> bool {
        self.system.connect_socket(&self.socket, None).is_ok()
    }

    pub fn kill(&mut self) -> Result<()> {
        match self.quit() {
            Err(Error::InstanceNotRunning(_)) => Ok(()),
            result => result,
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.execute("system_powerdown")
    }

    pub fn quit(&mut self) -> Result<()> {
        self.execute("quit")
    }

    pub fn add_hostfwd(&mut self, fwd: &PortForward) -> Result<()> {
        let output = self.run_hmp_command(&format!("hostfwd_add {NETDEV_ID} {}", fwd.to_qemu()))?;
        if output.is_empty() {
            Ok(())
        } else {
            Err(Error::HostfwdCommandFailed(output))
        }
    }

    pub fn remove_hostfwd(&mut self, fwd: &PortForward) -> Result<()> {
        let rule = format!(
            "{}:{}:{}",
            fwd.get_protocol(),
            fwd.get_host_ip(),
            fwd.get_host_port(),
        );
        let output = self.run_hmp_command(&format!("hostfwd_remove {NETDEV_ID} {rule}"))?;
        if output.contains("not found") {
            Err(Error::HostfwdCommandFailed(output))
        } else {
            Ok(())
        }
    }

    // Connects on first use, so a client exists while the instance is down
    fn get_stream(&mut self) -> Result<&mut BufReader<Box<dyn Socket>>> {
        if self.stream.is_none() {
            // A failed connect means the instance is down
            let socket = self
                .system
                .connect_socket(&self.socket, Some(QMP_TIMEOUT))
                .map_err(|_| Error::InstanceNotRunning(self.name.clone()))?;
            self.stream = Some(BufReader::new(socket));
            self.init()?;
        }
        Ok(self.stream.as_mut().unwrap())
    }

    // Drops the greeting, then negotiates capabilities, which QMP demands
    // before it accepts anything else.
    fn init(&mut self) -> Result<()> {
        self.recv()?;
        self.execute("qmp_capabilities")
    }

    // Runs an HMP command line through the QMP passthrough verb, for commands
    // with no native QMP equivalent. Returns the raw text QEMU printed, since
    // what counts as success differs per command.
    fn run_hmp_command(&mut self, command_line: &str) -> Result<String> {
        let response = self.execute_with_args(
            "human-monitor-command",
            json!({ "command-line": command_line }),
        )?;
        match response {
            QmpMessage::Success { ret, .. } => {
                Ok(ret.as_str().unwrap_or_default().trim().to_string())
            }
            QmpMessage::Error { error, .. } => Err(Error::HostfwdCommandFailed(error.desc)),
            _ => Ok(String::new()),
        }
    }

    fn send(&mut self, message: &QmpMessage) -> Result<()> {
        let request = serde_json::to_string(message).map_err(Error::from)?;
        let stream = self.get_stream()?.get_mut();
        stream.write_all(request.as_bytes()).map_err(Error::from)?;
        stream.flush().map_err(Error::from)
    }

    fn recv(&mut self) -> Result<QmpMessage> {
        let mut response = String::new();
        let stream = self.get_stream()?;
        stream.read_line(&mut response).map_err(Error::from)?;

        serde_json::from_str(&response).map_err(Error::from)
    }

    fn execute_with_args(&mut self, cmd: &str, arguments: Value) -> Result<QmpMessage> {
        // Connects first, so the handshake takes its id ahead of this request
        self.get_stream()?;
        let request_id = Some(self.counter.to_string());
        self.counter += 1;

        self.send(&QmpMessage::Command {
            id: request_id.clone(),
            execute: cmd.to_string(),
            arguments,
        })?;

        // Events and replies to earlier requests can arrive first, so skip
        // everything until this request's own id comes back.
        loop {
            let response = self.recv()?;
            match &response {
                QmpMessage::Success { id, .. } | QmpMessage::Error { id, .. }
                    if *id == request_id =>
                {
                    return Ok(response);
                }
                _ => {}
            }
        }
    }

    fn execute(&mut self, cmd: &str) -> Result<()> {
        self.execute_with_args(cmd, Value::Null).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserName;
    use crate::platform::SystemMock;
    use std::str::FromStr;

    const HANDSHAKE: &str = r#"{"QMP": {"version": {}, "capabilities": []}}
{"return": {}, "id": "0"}
"#;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".into(),
            "/cache".into(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    fn build_system(env: &Environment, replies: &str) -> Arc<SystemMock> {
        Arc::new(SystemMock::new().add_socket(
            env.get_monitor_socket("test"),
            format!("{HANDSHAKE}{replies}").as_bytes(),
        ))
    }

    #[test]
    fn test_is_running_follows_the_monitor_socket() {
        let env = build_env();
        let stopped = QemuMonitorClient::new(Arc::new(SystemMock::new()), &env, &build_instance());
        let running = QemuMonitorClient::new(build_system(&env, ""), &env, &build_instance());

        assert!(!stopped.is_running());
        assert!(running.is_running());
    }

    #[test]
    fn test_kill_is_ok_when_not_running() {
        let system = Arc::new(SystemMock::new());
        let mut client = QemuMonitorClient::new(system, &build_env(), &build_instance());

        assert!(client.kill().is_ok());
    }

    #[test]
    fn test_kill_quits_over_the_monitor() {
        let env = build_env();
        let system = build_system(&env, "{\"return\": {}, \"id\": \"1\"}\n");

        QemuMonitorClient::new(system.clone(), &env, &build_instance())
            .kill()
            .unwrap();

        assert!(
            system
                .get_socket_output(env.get_monitor_socket("test"))
                .ends_with(r#"{"id":"1","execute":"quit"}"#)
        );
    }

    #[test]
    fn test_shutdown_negotiates_capabilities_and_skips_an_event_before_the_reply() {
        let env = build_env();
        let system = build_system(
            &env,
            r#"{"event": "POWERDOWN", "timestamp": {"seconds": 1, "microseconds": 2}}
{"return": {}, "id": "1"}
"#,
        );

        QemuMonitorClient::new(system.clone(), &env, &build_instance())
            .shutdown()
            .unwrap();

        assert_eq!(
            system.get_socket_output(env.get_monitor_socket("test")),
            r#"{"id":"0","execute":"qmp_capabilities"}{"id":"1","execute":"system_powerdown"}"#
        );
    }

    #[test]
    fn test_add_hostfwd_reports_the_text_qemu_prints() {
        let env = build_env();
        let system = build_system(
            &env,
            "{\"return\": \"Could not set up host forwarding rule\\r\\n\", \"id\": \"1\"}\n",
        );

        let mut client = QemuMonitorClient::new(system, &env, &build_instance());

        assert!(matches!(
            client.add_hostfwd(&"127.0.0.1:4000:40/tcp".parse().unwrap()),
            Err(Error::HostfwdCommandFailed(output)) if output == "Could not set up host forwarding rule"
        ));
    }
}
