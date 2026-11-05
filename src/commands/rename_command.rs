use crate::actions::{ClaimInstanceNameAction, LoadInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::InstanceName;
use crate::util::ResourceLock;
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
        let names = [self.old_name.as_str(), self.new_name.as_str()];
        let _locks = ResourceLock::acquire_instances(context, &names)?;
        let instance = LoadInstanceAction::new().run(context, self.old_name.as_str())?;
        if context.get_monitor(&instance).is_running() {
            return Err(Error::InstanceNotStopped(instance.name.to_string()));
        }

        ClaimInstanceNameAction::new().run(context, self.new_name.as_str())?;
        let env = context.get_env();
        let old_dir = env.get_instance_dir2(&instance.name);
        let new_dir = env.get_instance_dir2(self.new_name.as_str());
        context.get_system().rename_file(&old_dir, &new_dir)?;
        Ok(0)
    }
}
