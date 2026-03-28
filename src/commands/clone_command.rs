use crate::actions::CreateInstanceAction;
use crate::commands::Command;
use crate::env::Environment;
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::image::ImageStore;
use crate::instance::{InstanceName, InstanceStore};
use crate::ssh_cmd::PortChecker;
use crate::view::{Console, SpinnerView};
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
pub struct CloneCommand {
    /// Name of the virtual machine instance to clone
    name: InstanceName,
    /// Name of the copy
    new_name: InstanceName,
}

impl Command for CloneCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<()> {
        // Verify that the target name is available
        if instance_store.exists(self.new_name.as_str()) {
            return Err(Error::InstanceAlreadyExists(self.new_name.to_string()));
        }

        // Load source instance info
        let source = &instance_store.load(self.name.as_str())?;

        // Verify that the source instance is stopped
        if instance_store.is_running(source) {
            return Err(Error::InstanceNotStopped(source.name.to_string()));
        }

        let mut spinner = SpinnerView::new("Cloning VM instance".to_string());

        // Load source instance info
        let image_path = &env.get_instance_image_file(self.name.as_str());

        // Setup target instance info
        let mut target = source.clone();
        target.name = self.new_name.to_string();
        target.ssh_port = PortChecker::new().get_new_port();

        // Create VM instance
        CreateInstanceAction::new().run(env, &FS::new(), instance_store, image_path, target)?;

        spinner.stop();
        Ok(())
    }
}
