use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
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

impl Command for InstanceRunCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let instance_name = self.create_cmd.get_name()?;

        self.create_cmd
            .run(console, env, image_store, instance_store)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(instance_name.clone()),
            xforward: false,
            ssh_args: None,
            cmd: None,
            openssh: self.openssh,
            russh: self.russh,
        }
        .run(console, env, image_store, instance_store)
    }
}
