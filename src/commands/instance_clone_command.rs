use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;

pub struct InstanceCloneCommand {
    name: String,
    new_name: String,
}

impl InstanceCloneCommand {
    pub fn new(name: &str, new_name: &str) -> Self {
        Self {
            name: name.to_string(),
            new_name: new_name.to_string(),
        }
    }

    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        instance_dao.clone(&instance_dao.load(&self.name)?, &self.new_name)?;

        let mut new_instance = instance_dao.load(&self.new_name)?;
        new_instance.ssh_port = util::generate_random_ssh_port();
        instance_dao.store(&new_instance)
    }
}
