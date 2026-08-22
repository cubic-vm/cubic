use crate::actions::{CreateInstanceAction, LoadInstanceAction};
use crate::commands::{Command, Context};
use crate::error::{Error, Result};
use crate::models::{InstanceName, LOW_DISK_SPACE_WARNING, ResourceAllocator};
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
    fn run(&self, console: &mut Console<'_>, context: &Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        // Verify that the target name is available
        if instance_store.exists(self.new_name.as_str()) {
            return Err(Error::InstanceAlreadyExists(self.new_name.to_string()));
        }

        if ResourceAllocator::is_disk_space_low(context.get_system(), context.get_env()) {
            console.warn(LOW_DISK_SPACE_WARNING);
        }

        // Load source instance info
        let source = &LoadInstanceAction::new().run(context, console, self.name.as_str())?;

        // Verify that the source instance is stopped
        if instance_store.is_running(source) {
            return Err(Error::InstanceNotStopped(source.name.to_string()));
        }

        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Cloning {} to {}",
            self.name, self.new_name
        )))));

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
        CreateInstanceAction::new().run(context, image_path, target)?;

        console.stop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::Environment;
    use crate::models::Instance;
    use crate::models::UserName;
    use crate::platform::SystemMock;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[test]
    fn test_clone_rejects_existing_target_name() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(vec![
            Instance {
                name: "test".to_string(),
                ..Instance::default()
            },
            Instance {
                name: "test2".to_string(),
                ..Instance::default()
            },
        ]);

        let result = CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("test2").unwrap(),
        }
        .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::InstanceAlreadyExists(ref name)) if name == "test2"
        ));
    }

    #[test]
    fn test_clone_rejects_running_source() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        let context = Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(InstanceStoreMock::new_with_running(
                vec![Instance {
                    name: "test".to_string(),
                    ..Instance::default()
                }],
                &["test"],
            )),
        );

        let result = CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("newname").unwrap(),
        }
        .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::InstanceNotStopped(ref name)) if name == "test"
        ));
    }

    #[test]
    fn test_clone_drops_the_host_key_of_the_source() {
        let source_image = PathBuf::from("machines")
            .join("test")
            .join("machine.img")
            .to_string_lossy()
            .into_owned();
        // The action builds the temporary image path itself, so the expected
        // command has to be spelled the same way.
        let target_dir = PathBuf::from("machines")
            .join("test2")
            .to_string_lossy()
            .into_owned();
        let target_image = format!("{target_dir}.tmp/machine.img");
        let system = SystemMock::new()
            .add_command_output(
                &format!("qemu-img convert -f qcow2 -O qcow2 {source_image} {target_image}"),
                b"",
            )
            .add_command_output(&format!("qemu-img resize {target_image} 0"), b"");
        let console_system = SystemMock::new();
        let console = &mut Console::new(&console_system);
        let store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            ssh_host_key: Some("ssh-ed25519 AAAA".to_string()),
            ..Instance::default()
        }]);
        let stored = Arc::clone(&store.stored);
        let context = Context::new(
            Rc::new(system),
            Environment::new(
                UserName::from_str("cubic").unwrap(),
                String::new(),
                String::new(),
            ),
            Box::new(store),
        );

        CloneCommand {
            name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("test2").unwrap(),
        }
        .run(console, &context)
        .unwrap();

        let stored = stored.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].ssh_host_key, None);
    }

    #[test]
    fn test_clone_rejects_unknown_source() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        let result = CloneCommand {
            name: InstanceName::from_str("missing").unwrap(),
            new_name: InstanceName::from_str("newname").unwrap(),
        }
        .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }
}
