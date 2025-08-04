use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use clap::Parser;

/// Rename a virtual machine instance
#[derive(Parser)]
pub struct InstanceRenameCommand {
    /// Name of the virtual machine instance to rename
    old_name: String,
    /// New name of the virutal machine instance
    new_name: String,
}

impl InstanceRenameCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.rename(&mut instance_dao.load(&self.old_name)?, &self.new_name)
    }
}
