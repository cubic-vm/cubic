use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::InstanceDao;
use crate::util;

pub struct InstanceRemoveCommand {
    verbosity: Verbosity,
    force: bool,
    instances: Vec<String>,
}

impl InstanceRemoveCommand {
    pub fn new(verbosity: Verbosity, force: bool, instances: &[String]) -> Self {
        Self {
            verbosity,
            force,
            instances: instances.to_vec(),
        }
    }

    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        if self.force {
            commands::stop(instance_dao, false, self.verbosity, &self.instances)?;
        }

        for instance in &self.instances {
            if !instance_dao.exists(instance) {
                return Result::Err(Error::UnknownInstance(instance.clone()));
            }

            if instance_dao.is_running(&instance_dao.load(instance)?) {
                return Result::Err(Error::InstanceNotStopped(instance.to_string()));
            }
        }

        for instance in &self.instances {
            if util::confirm(&format!(
                "Do you really want delete the instance '{instance}'? [y/n]: "
            )) {
                instance_dao.delete(&instance_dao.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Result::Ok(())
    }
}
