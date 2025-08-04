use crate::commands;
use crate::error::Error;
use crate::instance::InstanceDao;
use clap::Parser;

/// Restart virtual machine instances
#[derive(Parser)]
pub struct InstanceRestartCommand {
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
    /// Name of the virtual machine instances to restart
    instances: Vec<String>,
}

impl InstanceRestartCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        commands::InstanceStopCommand {
            all: false,
            verbose: self.verbose,
            quiet: self.quiet,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(instance_dao)?;
        commands::InstanceStartCommand {
            qemu_args: None,
            verbose: self.verbose,
            quiet: self.quiet,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(instance_dao)
    }
}
