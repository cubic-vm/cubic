use crate::commands::{self, Command};
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, Target};
use crate::view::Console;
use clap::{self, Parser};

/// Create and start VM instances
///
/// This command is a shortcut for the three subcommands `create`, `start` and `ssh`.
///
/// Examples:
///
///   Run a VM instance with 8 vCPUs, 10G of RAM, 200G of storage:
///   $ cubic run example1 --cpus 8 --mem 10G --disk 200G -i debian:trixie
///
///   Run a VM instance and forward the instance's HTTP port to the host port 8000:
///   $ cubic run example2 --port 8000:80 -i ubuntu:noble
///
///   Run a VM instance and forward the instance's DNS port to the host port 5353:
///   $ cubic run example3 --port 5353:53/udp -i ubuntu:noble
///
///   Run a VM instance with multiple port forwarding rules:
///   $ cubic run example4 -p 8000:80/tcp -p 5353:53/udp -i ubuntu:noble
///
///   Run a VM instance and install Vim:
///   $ cubic run example5 -e "sudo apt install -y vim" -i ubuntu:noble
///
///   Run a VM instance without network access:
///   $ cubic run example6 --isolate ubuntu:noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct InstanceRunCommand {
    #[clap(flatten)]
    create_cmd: commands::CreateInstanceCommand,
}

impl Command for InstanceRunCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        self.create_cmd
            .run(console, env, image_store, instance_store)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(self.create_cmd.instance_name.clone()),
            cmd: None,
        }
        .run(console, env, image_store, instance_store)
    }
}
