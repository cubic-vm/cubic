use crate::commands;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;
use clap::Parser;

/// Delete virtual machine instances
#[derive(Parser)]
pub struct InstanceRemoveCommand {
    /// Delete the virtual machine instances even when running
    #[clap(short, long, default_value_t = false)]
    force: bool,
    /// Delete the virtual machine instances without confirmation
    #[clap(short, long, default_value_t = false)]
    yes: bool,
    /// Name of the virtual machine instances to delete
    instances: Vec<String>,
}

impl InstanceRemoveCommand {
    pub fn run(
        &self,
        instance_dao: &InstanceDao,
        verbosity: commands::Verbosity,
    ) -> Result<(), Error> {
        if self.force {
            commands::InstanceStopCommand {
                all: false,
                wait: true,
                instances: self.instances.clone(),
            }
            .run(instance_dao, verbosity)?;
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
            if self.yes
                || util::confirm(&format!(
                    "Do you really want delete the instance '{instance}'? [y/n]: "
                ))
            {
                instance_dao.delete(&instance_dao.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Result::Ok(())
    }
}
