use crate::commands::{self, Command, Iso9660Arg};
use crate::error::Result;
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
    #[clap(flatten)]
    pub iso9660: Iso9660Arg,
}

impl Command for RestartCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        commands::StopCommand {
            all: false,
            wait: true,
            instances: self.instances.to_vec(),
        }
        .run(console, context)?;
        commands::StartCommand {
            qemu_args: None,
            wait: true,
            instances: self.instances.to_vec(),
            iso9660: self.iso9660.clone(),
        }
        .run(console, context)
    }
}
