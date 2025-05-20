use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};

pub struct InstanceRenameCommand {
    old_name: String,
    new_name: String,
}

impl InstanceRenameCommand {
    pub fn new(old_name: &str, new_name: &str) -> Self {
        Self {
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        }
    }

    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.rename(&mut instance_dao.load(&self.old_name)?, &self.new_name)
    }
}
