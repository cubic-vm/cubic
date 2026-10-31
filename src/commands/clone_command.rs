use crate::actions::{CreateInstanceAction, LoadInstanceAction};
use crate::commands::{Command, Context};
use crate::error::{Error, Result};
use crate::models::{InstanceName, LOW_DISK_SPACE_WARNING, ResourceAllocator};
use crate::view::Spinner;
use clap::Parser;
use std::sync::Arc;

/// Clone a VM instance
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
    async fn run(&self, context: &Context) -> Result<u8> {
        let console = context.get_console();
        let instance_store = context.get_instance_store();

        if ResourceAllocator::is_disk_space_low(context.get_system(), context.get_env()) {
            console.warn(LOW_DISK_SPACE_WARNING);
        }

        // Load source instance info
        let source = &LoadInstanceAction::new().run(context, self.name.as_str())?;

        // Verify that the source instance is stopped
        if instance_store.is_running(source) {
            return Err(Error::InstanceNotStopped(source.name.to_string()));
        }

        // Verify that the target name is available
        instance_store.claim_name(self.new_name.as_str())?;

        let text = format!("Cloning {} to {}", self.name, self.new_name);
        let _spinner = Spinner::new(Arc::clone(console), text);

        // Load source instance info
        let image_path = &context
            .get_env()
            .get_instance_image_file(self.name.as_str());

        // Setup target instance info
        let mut target = source.clone();
        target.name = self.new_name.to_string();
        target.ssh_port = context.get_system().bind_port()?;
        // The clone gets its own cloud-init seed, so the guest generates new
        // host keys on the first boot.
        target.ssh_host_key = None;

        // Create VM instance
        CreateInstanceAction::new().run(context, image_path, target, false)?;

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::{Environment, Instance};
    use crate::platform::SystemMock;
    use std::str::FromStr;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_clone_rejects_existing_target_name() {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![
            Instance {
                name: "test".to_string(),
                ..Instance::default()
            },
            Instance {
                name: "test2".to_string(),
                ..Instance::default()
            },
        ]);
        let context = builder.build();

        let result = CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("test2").unwrap(),
        }
        .run(&context)
        .await;

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test2"
        ));
    }

    #[tokio::test]
    async fn test_clone_rejects_running_source() {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        builder.get_store().set_running(&["test"]);
        let context = builder.build();

        let result = CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("newname").unwrap(),
        }
        .run(&context)
        .await;

        assert!(matches!(
            result,
            Err(Error::InstanceNotStopped(ref name)) if name == "test"
        ));
    }

    #[tokio::test]
    async fn test_clone_drops_the_host_key_of_the_source() {
        let env = Environment::default();
        let source_image = env.get_instance_image_file("test");
        let target_image = env.get_instance_image_file("test2.tmp");
        let resize = format!("qemu-img resize {} 0", target_image.display());
        let system = SystemMock::new()
            .add_file(source_image, b"qcow2 image")
            .add_command_output(&resize, b"");
        let mut builder = ContextBuilder::new(system);
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            ssh_host_key: Some("ssh-ed25519 AAAA".to_string()),
            ..Instance::default()
        }]);
        let stored = Arc::clone(&builder.get_store().stored);
        let context = builder.build();

        CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("test2").unwrap(),
        }
        .run(&context)
        .await
        .unwrap();

        let stored = stored.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].ssh_host_key, None);
    }
}
