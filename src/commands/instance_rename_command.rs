use crate::commands::Command;
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::{InstanceName, InstanceStore};
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
pub struct InstanceRenameCommand {
    /// Name of the virtual machine instance to rename
    old_name: InstanceName,
    /// New name of the virtual machine instance
    new_name: InstanceName,
}

impl Command for InstanceRenameCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        instance_store.rename(
            &mut instance_store.load(self.old_name.as_str())?,
            self.new_name.as_str(),
        )
    }
}
