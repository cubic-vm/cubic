use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::view::{ConfirmDialog, Console};
use clap::Parser;

/// Delete VM instances
///
/// Examples:
///
///   Delete the VM instance 'my-instance':
///   $ cubic delete my-instance
///
///   Delete multiple VM instances:
///   $ cubic delete trixie noble
///
///   Delete multiple VM instances without confirmation:
///   $ cubic delete --yes trixie noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct DeleteCommand {
    /// Delete the VM instances even when running (Deprecated)
    #[clap(hide = true, short, long, default_value_t = false)]
    force: bool,
    #[clap(flatten)]
    yes: commands::YesArg,
    #[clap(flatten)]
    instances: commands::InstancesArg,
}

impl Command for DeleteCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        self.instances.require_names()?;

        // Check if the instance names are valid
        for instance in &self.instances.value {
            if !instance_store.exists(instance.as_str()) {
                return Err(Error::UnknownInstance(instance.to_string()));
            }
        }

        // Print instances to be deleted
        console.print("The following VM instances are going to be deleted:");
        for instance in &self.instances.value {
            console.print(&format!("  - {instance}"));
        }

        // Ask for confirmation
        if self.yes.value || ConfirmDialog::new("\nDo you want to proceed?").confirm(console) {
            // Stop the VM instances
            commands::StopCommand {
                all: false,
                wait: true,
                kill: false,
                instances: self.instances.value.clone().into(),
            }
            .run(console, context)?;

            // Delete the VM instances
            for instance in &self.instances.value {
                instance_store.delete(&instance_store.load(instance.as_str())?)?;
                console.print(&format!("Deleted instance {instance}"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    #[test]
    fn test_reject_path_traversal() {
        assert!(DeleteCommand::try_parse_from(["delete", "../../etc"]).is_err());
    }

    #[test]
    fn test_reject_empty_instance_list() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("myuser").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        let context = commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(ImageStoreMock::default()),
            Box::new(InstanceStoreMock::new(Vec::new())),
        );

        assert!(matches!(
            DeleteCommand {
                force: false,
                yes: commands::YesArg { value: true },
                instances: Vec::new().into(),
            }
            .run(console, &context),
            Err(Error::MissingInstanceName)
        ));
    }
}
