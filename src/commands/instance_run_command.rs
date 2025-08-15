use crate::commands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{InstanceDao, Target};
use clap::Parser;

/// Create, start and open a shell in a new virtual machine instance
#[derive(Parser)]
pub struct InstanceRunCommand {
    #[clap(flatten)]
    add_cmd: commands::InstanceAddCommand,
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
}

impl InstanceRunCommand {
    pub fn run(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        let instance_name = self.add_cmd.get_name()?;

        self.add_cmd.run(image_dao, instance_dao)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(instance_name.clone()),
            xforward: false,
            verbose: self.verbose,
            quiet: self.quiet,
            ssh_args: None,
            cmd: None,
        }
        .run(instance_dao)
    }
}
