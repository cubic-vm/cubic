use crate::error::Error;
use crate::instance::{InstanceDao, InstanceName, InstanceStore};
use crate::util;
use clap::Parser;

/// Clone a virtual machine instance
#[derive(Parser)]
pub struct InstanceCloneCommand {
    /// Name of the virtual machine instance to clone
    name: InstanceName,
    /// Name of the copy
    new_name: InstanceName,
}

impl InstanceCloneCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.clone(
            &instance_dao.load(self.name.as_str())?,
            self.new_name.as_str(),
        )?;

        let mut new_instance = instance_dao.load(self.new_name.as_str())?;
        new_instance.ssh_port = util::generate_random_ssh_port();
        instance_dao.store(&new_instance)
    }
}
