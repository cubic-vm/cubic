use crate::commands;
use crate::error::Error;
use crate::instance::InstanceDao;
use clap::Parser;

/// Restart virtual machine instances
#[derive(Parser)]
pub struct InstanceRestartCommand {
    /// Name of the virtual machine instances to restart
    instances: Vec<String>,
}

impl InstanceRestartCommand {
    pub fn run(
        &self,
        instance_dao: &InstanceDao,
        verbosity: commands::Verbosity,
    ) -> Result<(), Error> {
        commands::InstanceStopCommand {
            all: false,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(instance_dao, verbosity)?;
        commands::InstanceStartCommand {
            qemu_args: None,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(instance_dao, verbosity)
    }
}
