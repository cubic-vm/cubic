use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::InstanceName;
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
    fn run(&self, _console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        instance_store.rename(
            &mut instance_store.load(self.old_name.as_str())?,
            self.new_name.as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_context(instances: Vec<Instance>) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(instances)),
        )
    }

    #[test]
    fn test_rename_rejects_unknown_instance() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(Vec::new());

        let result = RenameCommand {
            old_name: InstanceName::from_str("missing").unwrap(),
            new_name: InstanceName::from_str("newname").unwrap(),
        }
        .run(console, &context);

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_rename_delegates_to_store() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let context = build_context(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);

        let result = RenameCommand {
            old_name: InstanceName::from_str("test").unwrap(),
            new_name: InstanceName::from_str("newname").unwrap(),
        }
        .run(console, &context);

        assert!(result.is_ok());
    }
}
