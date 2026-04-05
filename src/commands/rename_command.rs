use crate::commands::{self, Command};
use crate::error::Result;
use crate::instance::InstanceName;
use crate::view::Console;
use clap::Parser;

/// Rename VM instances
///
/// Examples:
///
///   Rename the VM instance 'noble' in 'ubuntu':
///   $ cubic rename noble ubuntu
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RenameCommand {
    /// Name of the virtual machine instance to rename
    old_name: InstanceName,
    /// New name of the virtual machine instance
    new_name: InstanceName,
}

impl Command for RenameCommand {
    fn run(&self, _console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        instance_store.rename(
            &mut instance_store.load(self.old_name.as_str())?,
            self.new_name.as_str(),
        )
    }
}
