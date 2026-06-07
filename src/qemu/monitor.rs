use crate::commands::Verbosity;
use crate::error::{Error, Result};
use crate::models::{Environment, Instance, InstanceCertPaths};
use crate::qemu::{Qmp, TlsClient};
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
}
