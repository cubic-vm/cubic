#[cfg(feature = "russh")]
use crate::ssh_cmd::Russh;
use crate::ssh_cmd::{Openssh, Ssh};

#[derive(Default)]
pub struct SshFactory;

impl SshFactory {
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "russh")]
    pub fn create(&self, use_russh: bool) -> Box<dyn Ssh> {
        if use_russh {
            Box::new(Russh::new())
        } else {
            Box::new(Openssh::new())
        }
    }

    #[cfg(not(feature = "russh"))]
    pub fn create(&self, _use_russh: bool) -> Box<dyn Ssh> {
        Box::new(Openssh::new())
    }
}
