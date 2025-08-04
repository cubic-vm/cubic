use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;
use clap::Parser;

/// Delete virtual machine instances
#[derive(Parser)]
pub struct InstanceRemoveCommand {
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
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
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let verbosity = Verbosity::new(self.verbose, self.quiet);
        if self.force {
            commands::stop(instance_dao, false, verbosity, true, &self.instances)?;
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
