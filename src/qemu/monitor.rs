use crate::commands::Verbosity;
use crate::env::Environment;
use crate::error::Result;
use crate::qemu::Qmp;

pub struct Monitor {
    qmp: Qmp,
}

impl Monitor {
    pub fn new(env: &Environment, instance: &str) -> Result<Self> {
        let mut monitor = Monitor {
            qmp: Qmp::new(&env.get_monitor_file(instance), Verbosity::Normal)?,
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
