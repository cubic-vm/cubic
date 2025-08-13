use crate::error::Error;
use crate::instance::{InstanceDao, InstanceName, InstanceStore};
use clap::Parser;

/// Rename a virtual machine instance
#[derive(Parser)]
pub struct InstanceRenameCommand {
    /// Name of the virtual machine instance to rename
    old_name: InstanceName,
    /// New name of the virtual machine instance
    new_name: InstanceName,
}

impl InstanceRenameCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.rename(
            &mut instance_dao.load(self.old_name.as_str())?,
            self.new_name.as_str(),
        )
    }
}
