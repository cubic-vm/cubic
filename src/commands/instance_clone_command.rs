use crate::commands::Command;
use crate::env::Environment;
use crate::error::Result;
use crate::image::ImageStore;
use crate::instance::{InstanceName, InstanceStore};
use crate::ssh_cmd::PortChecker;
use crate::view::Console;
use clap::Parser;

/// Clone VM instances
///
/// Examples:
///
///   Clone the VM instance 'my-instance' as 'my-instance2':
///   $ cubic clone my-instance my-instance2
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct InstanceCloneCommand {
    /// Name of the virtual machine instance to clone
    name: InstanceName,
    /// Name of the copy
    new_name: InstanceName,
}

impl Command for InstanceCloneCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        instance_store.clone(
            &instance_store.load(self.name.as_str())?,
            self.new_name.as_str(),
        )?;

        let mut new_instance = instance_store.load(self.new_name.as_str())?;
        new_instance.ssh_port = PortChecker::new().get_new_port();
        instance_store.store(&new_instance)
    }
}
