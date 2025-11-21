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
    #[clap(flatten)]
    ssh_args: commands::SshArgs,
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
            args: self.ssh_args.clone(),
            ssh_args: None,
            target: Target::from_instance_name(instance_name.clone()),
            cmd: None,
        }
        .run(console, env, image_store, instance_store)
    }
}
