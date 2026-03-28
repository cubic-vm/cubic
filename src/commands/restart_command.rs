use crate::commands::{self, Command, Iso9660};
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
use clap::Parser;

/// Restart VM instances
///
/// Examples:
///
///   Restart the VM instance 'my-instance':
///   $ cubic restart my-instance
///
///   Restart multiple VM instances:
///   $ cubic restart trixie noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RestartCommand {
    /// Name of the virtual machine instances to restart
    instances: Vec<String>,
    /// Switch for Rust and system ISO9600 implementation
    #[clap(hide = true)]
    #[arg(value_enum, long, default_value_t = Iso9660::System)]
    pub iso9660: Iso9660,
}

impl Command for RestartCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        commands::StopCommand {
            all: false,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(console, env, image_store, instance_store)?;
        commands::StartCommand {
            qemu_args: None,
            wait: true,
            instances: self.instances.to_vec(),
            iso9660: self.iso9660.clone(),
        }
        .run(console, env, image_store, instance_store)
    }
}
