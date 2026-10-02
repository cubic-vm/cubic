use crate::error::{Error, Result};
use crate::models::{Environment, Instance, PortForward};
use crate::platform::{Socket, System};
use crate::qemu::{NETDEV_ID, QmpMessage};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;

const QMP_TIMEOUT: Duration = Duration::from_millis(100);

pub struct QemuMonitorClient {
    counter: u64,
    stream: BufReader<Box<dyn Socket>>,
}

impl QemuMonitorClient {
    pub fn new(system: &dyn System, env: &Environment, instance: &Instance) -> Result<Self> {
        // A failed connect means the instance is down
        let socket = system
            .connect_socket(
                Path::new(&env.get_monitor_socket(&instance.name)),
                Some(QMP_TIMEOUT),
            )
            .map_err(|_| Error::InstanceNotRunning(instance.name.clone()))?;

        let mut client = QemuMonitorClient {
            counter: 0,
            stream: BufReader::new(socket),
        };
        client.init()?;
        Ok(client)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.execute("system_powerdown")
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
        let stream = self.stream.get_mut();
        stream.write_all(request.as_bytes()).map_err(Error::from)?;
        stream.flush().map_err(Error::from)
    }

    fn recv(&mut self) -> Result<QmpMessage> {
        let mut response = String::new();
        self.stream.read_line(&mut response).map_err(Error::from)?;

        serde_json::from_str(&response).map_err(Error::from)
    }

    fn execute_with_args(&mut self, cmd: &str, arguments: Value) -> Result<QmpMessage> {
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
            "/data".to_string(),
            "/cache".to_string(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    fn build_system(env: &Environment, replies: &str) -> SystemMock {
        SystemMock::new().add_socket(
            &env.get_monitor_socket("test"),
            format!("{HANDSHAKE}{replies}").as_bytes(),
        )
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

        QemuMonitorClient::new(&system, &env, &build_instance())
            .unwrap()
            .shutdown()
            .unwrap();

        assert_eq!(
            system.get_socket_output(&env.get_monitor_socket("test")),
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

        let mut client = QemuMonitorClient::new(&system, &env, &build_instance()).unwrap();

        assert!(matches!(
            client.add_hostfwd(&"127.0.0.1:4000:40/tcp".parse().unwrap()),
            Err(Error::HostfwdCommandFailed(output)) if output == "Could not set up host forwarding rule"
        ));
    }
}
