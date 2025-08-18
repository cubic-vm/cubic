use crate::commands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{InstanceDao, Target};
use crate::view::Console;
use clap::Parser;

/// Create, start and open a shell in a new virtual machine instance
#[derive(Parser)]
pub struct InstanceRunCommand {
    #[clap(flatten)]
    add_cmd: commands::InstanceAddCommand,
}

impl InstanceRunCommand {
    pub fn run(
        &self,
        console: &mut dyn Console,
        image_dao: &ImageDao,
        instance_dao: &InstanceDao,
        verbosity: commands::Verbosity,
    ) -> Result<(), Error> {
        let instance_name = self.add_cmd.get_name()?;

        self.add_cmd.run(console, image_dao, instance_dao)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(instance_name.clone()),
            xforward: false,
            ssh_args: None,
            cmd: None,
        }
        .run(console, instance_dao, verbosity)
    }
}
