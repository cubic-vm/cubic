use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::InstanceName;
use clap::Parser;

/// Rename a VM instance
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
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let instance_store = context.get_instance_store();

        let mut instance = LoadInstanceAction::new().run(context, self.old_name.as_str())?;
        instance_store.rename(&mut instance, self.new_name.as_str())?;
        Ok(0)
    }
}
