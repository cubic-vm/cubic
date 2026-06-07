use crate::commands::Verbosity;
use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::qemu::Qmp;

pub struct Monitor {
    qmp: Qmp,
}

impl Monitor {
    pub fn new(_env: &Environment, instance: &Instance) -> Result<Self> {
        let port = instance
            .monitor_port
            .ok_or_else(|| Error::InstanceNotRunning(instance.name.clone()))?;
        let mut monitor = Monitor {
            qmp: Qmp::new(port, Verbosity::Normal)?,
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
