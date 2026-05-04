use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::util;
use crate::view::Console;
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
    /// Name of the VM instances to delete
    instances: Vec<String>,
}

impl Command for DeleteCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        // Check if the instance names are valid
        for instance in &self.instances {
            if !instance_store.exists(instance) {
                return Result::Err(Error::UnknownInstance(instance.clone()));
            }
        }

        // Print instances to be deleted
        console.info("The following VM instances are going to be deleted:");
        for instance in &self.instances {
            console.info(&format!("  - {instance}"));
        }

        // Ask for confirmation
        if self.yes.value || util::confirm("\nDo you want to proceed? [y/n]: ") {
            // Stop the VM instances
            commands::StopCommand {
                all: false,
                wait: true,
                instances: self.instances.clone(),
            }
            .run(console, context)?;

            // Delete the VM instances
            for instance in &self.instances {
                instance_store.delete(&instance_store.load(instance)?)?;
                println!("Deleted instance {instance}");
            }
        }

        Ok(())
    }
}
