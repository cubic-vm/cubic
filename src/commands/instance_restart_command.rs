use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::Parser;

/// Restart virtual machine instances
#[derive(Parser)]
pub struct InstanceRestartCommand {
    /// Name of the virtual machine instances to restart
    instances: Vec<String>,
}

impl Command for InstanceRestartCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        commands::InstanceStopCommand {
            all: false,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(console, env, image_store, instance_store)?;
        commands::InstanceStartCommand {
            qemu_args: None,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(console, env, image_store, instance_store)
    }
}
