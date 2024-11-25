use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu::Qmp;

pub struct Monitor {
    qmp: Qmp,
}

impl Monitor {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut monitor = Monitor {
            qmp: Qmp::new(path, Verbosity::Normal)?,
        };
        monitor.init()?;
        Ok(monitor)
    }

    fn init(&mut self) -> Result<(), Error> {
        self.qmp.recv().map(|_| ())?;
        self.qmp.execute("qmp_capabilities").map(|_| ())
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.qmp.execute("system_powerdown")
    }
}
