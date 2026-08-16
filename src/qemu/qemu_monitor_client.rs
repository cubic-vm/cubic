use crate::error::{Error, Result};
use crate::models::{Environment, Instance, InstanceCertPaths, PortForward};
use crate::platform::ReadWrite;
use crate::qemu::{NETDEV_ID, QmpMessage, TlsClient};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;

const QMP_TIMEOUT: Duration = Duration::from_millis(100);

pub struct QemuMonitorClient {
    counter: u64,
    stream: BufReader<Box<dyn ReadWrite>>,
}

impl QemuMonitorClient {
    pub fn new(env: &Environment, instance: &Instance) -> Result<Self> {
        let port = instance
            .monitor_port
            .ok_or_else(|| Error::InstanceNotRunning(instance.name.clone()))?;
        let instance_dir = PathBuf::from(env.get_instance_dir2(&instance.name));
        let certs = InstanceCertPaths::load(&instance_dir);
        let mut stream = TlsClient::new(&certs)?.connect(port)?;
        let socket = stream.get_mut();
        socket
            .set_read_timeout(Some(QMP_TIMEOUT))
            .map_err(|e| Error::ConnectionFailed(port, e))?;
        socket
            .set_write_timeout(Some(QMP_TIMEOUT))
            .map_err(|e| Error::ConnectionFailed(port, e))?;

        let mut client = QemuMonitorClient {
            counter: 0,
            stream: BufReader::new(Box::new(stream)),
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
