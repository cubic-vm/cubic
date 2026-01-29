use crate::ssh_cmd::Russh;
use crate::ssh_cmd::{Openssh, Ssh};

#[derive(Default)]
pub struct SshFactory;

impl SshFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self, use_openssh: bool) -> Box<dyn Ssh> {
        if use_openssh {
            Box::new(Openssh::new())
        } else {
            Box::new(Russh::new())
        }
    }
}
