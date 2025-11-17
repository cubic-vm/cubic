use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::util;
use crate::view::Console;
use clap::Parser;

/// Delete one or more virtual machine instances
#[derive(Parser)]
pub struct DeleteInstanceCommand {
    /// Delete the virtual machine instances even when running
    #[clap(short, long, default_value_t = false)]
    force: bool,
    /// Delete the virtual machine instances without confirmation
    #[clap(short, long, default_value_t = false)]
    yes: bool,
    /// Name of the virtual machine instances to delete
    instances: Vec<String>,
}

impl Command for DeleteInstanceCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        if self.force {
            commands::InstanceStopCommand {
                all: false,
                wait: true,
                instances: self.instances.clone(),
            }
            .run(console, env, image_store, instance_store)?;
        }

        for instance in &self.instances {
            if !instance_store.exists(instance) {
                return Result::Err(Error::UnknownInstance(instance.clone()));
            }

            if instance_store.is_running(&instance_store.load(instance)?) {
                return Result::Err(Error::InstanceNotStopped(instance.to_string()));
            }
        }

        for instance in &self.instances {
            if self.yes
                || util::confirm(&format!(
                    "Do you really want delete the instance '{instance}'? [y/n]: "
                ))
            {
                instance_store.delete(&instance_store.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Result::Ok(())
    }
}
