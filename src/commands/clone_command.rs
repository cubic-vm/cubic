use crate::actions::CreateInstanceAction;
use crate::commands::{Command, Context};
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::models::InstanceName;
use crate::ssh_cmd::PortChecker;
use crate::view::{Console, Spinner};
use clap::Parser;
use std::sync::{Arc, Mutex};

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
    fn run(&self, console: &mut dyn Console, context: &Context) -> Result<()> {
        let instance_store = context.get_instance_store();

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

        console.play(Arc::new(Mutex::new(Spinner::new(
            "Cloning VM instance".to_string(),
        ))));

        // Load source instance info
        let image_path = &context
            .get_env()
            .get_instance_image_file(self.name.as_str());

        // Setup target instance info
        let mut target = source.clone();
        target.name = self.new_name.to_string();
        target.ssh_port = PortChecker::new().get_new_port()?;

        // Create VM instance
        CreateInstanceAction::new().run(context, &FS::new(), image_path, target)?;

        console.stop();
        Ok(())
    }
}
