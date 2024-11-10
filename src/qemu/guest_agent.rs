use crate::commands::Verbosity;
use crate::error::Error;
use crate::qemu::Qmp;

pub struct GuestAgent {
    qmp: Qmp,
}

impl GuestAgent {
    pub fn new(path: &str) -> Result<Self, Error> {
        Ok(GuestAgent {
            qmp: Qmp::new(path, Verbosity::Normal)?,
        })
    }

    pub fn ping(&mut self) -> Result<(), Error> {
        self.qmp.execute("guest-ping").map(|_| ())
    }
}
