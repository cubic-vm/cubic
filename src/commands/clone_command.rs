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
        target.ssh_port = PortChecker::new().get_new_port()?;

        // Create VM instance
        CreateInstanceAction::new().run(context, &FS::new(), image_path, target)?;

        console.stop();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::Environment;
    use crate::models::Instance;
    use crate::view::ConsoleMock;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> Context {
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        Context::new(
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[test]
    fn test_clone_rejects_existing_target_name() {
        let console = &mut ConsoleMock::new();
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
        let console = &mut ConsoleMock::new();
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let context = Context::new(
            env,
            Box::new(ImageStoreMock::default()),
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
    fn test_clone_rejects_unknown_source() {
        let console = &mut ConsoleMock::new();
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
