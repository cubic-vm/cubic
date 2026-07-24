use crate::commands::Verbosity;
use crate::error::{Error, Result};
use crate::models::{Environment, Instance, InstanceCertPaths, PortForward};
use crate::qemu::{NETDEV_ID, Qmp, QmpMessage, TlsClient};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;

const QMP_TIMEOUT_MS: u64 = 100;

pub struct Monitor {
    qmp: Qmp,
}

impl Monitor {
    pub fn new(env: &Environment, instance: &Instance) -> Result<Self> {
        let port = instance
            .monitor_port
            .ok_or_else(|| Error::InstanceNotRunning(instance.name.clone()))?;
        let instance_dir = PathBuf::from(env.get_instance_dir2(&instance.name));
        let certs = InstanceCertPaths::load(&instance_dir);
        let mut stream = TlsClient::new(&certs)?.connect(port)?;
        stream
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(QMP_TIMEOUT_MS)))
            .map_err(Error::from)?;
        stream
            .get_mut()
            .set_write_timeout(Some(Duration::from_millis(QMP_TIMEOUT_MS)))
            .map_err(Error::from)?;
        let mut monitor = Monitor {
            qmp: Qmp::new(stream, Verbosity::Normal),
        };
        monitor.init()?;
        Ok(monitor)
    }

    pub fn init(&mut self) -> Result<()> {
        self.qmp.recv().map(|_| ())?;
        self.qmp.execute("qmp_capabilities").map(|_| ())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.qmp.execute("system_powerdown")
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

    /// Runs an HMP (human monitor protocol) command line over QMP via the
    /// `human-monitor-command` passthrough verb, for HMP commands that have
    /// no native QMP equivalent. Returns the raw (trimmed) text QEMU printed,
    /// since success/failure text conventions differ per HMP command and
    /// must be interpreted by the caller.
    fn run_hmp_command(&mut self, command_line: &str) -> Result<String> {
        let response = self.qmp.execute_with_args(
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
}
