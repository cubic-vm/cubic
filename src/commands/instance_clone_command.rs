use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;
use clap::Parser;

/// Clone a virtual machine instance
#[derive(Parser)]
pub struct InstanceCloneCommand {
    /// Name of the virtual machine instance to clone
    name: String,
    /// Name of the copy
    new_name: String,
}

impl InstanceCloneCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.clone(&instance_dao.load(&self.name)?, &self.new_name)?;

        let mut new_instance = instance_dao.load(&self.new_name)?;
        new_instance.ssh_port = util::generate_random_ssh_port();
        instance_dao.store(&new_instance)
    }
}
