use crate::commands;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{InstanceDao, Target};
use crate::view::Console;
use clap::{self, Parser};

/// Create, start and open a shell in a new virtual machine instance
#[derive(Parser)]
pub struct InstanceRunCommand {
    #[clap(flatten)]
    create_cmd: commands::CreateInstanceCommand,
    #[clap(long, conflicts_with = "russh", default_value_t = true, hide = true)]
    pub openssh: bool,
    #[clap(long, conflicts_with = "openssh", default_value_t = false, hide = true)]
    pub russh: bool,
}

impl InstanceRunCommand {
    pub fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_dao: &ImageDao,
        instance_dao: &InstanceDao,
        verbosity: commands::Verbosity,
    ) -> Result<(), Error> {
        let instance_name = self.create_cmd.get_name()?;

        self.create_cmd.run(console, env, image_dao, instance_dao)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(instance_name.clone()),
            xforward: false,
            ssh_args: None,
            cmd: None,
            openssh: self.openssh,
            russh: self.russh,
        }
        .run(console, instance_dao, verbosity)
    }
}
