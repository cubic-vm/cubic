use crate::actions::StopInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::view::Console;
use crate::view::Spinner;
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Stop VM instances
///
/// Examples:
///
///   Stop the VM instance 'my-instance':
///   $ cubic stop my-instance
///
///   Stop and wait until the VM instance 'my-instance' has stopped:
///   $ cubic stop --wait my-instance
///
///   Stop all VM instances:
///   $ cubic stop --all --wait
///
///   Force-kill the VM instance 'my-instance':
///   $ cubic stop --kill my-instance
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct StopCommand {
    /// Stop all virtual machine instances
    #[clap(short, long, default_value_t = false)]
    pub all: bool,
    /// Wait for the virtual machine instance to be stopped
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Kill the virtual machine instance
    #[clap(short, long, default_value_t = false)]
    pub kill: bool,
    #[clap(flatten)]
    pub instances: commands::InstancesArg,
}

impl Command for StopCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        if !self.all {
            self.instances.require_names()?;
        }

        let stop_instances = if self.all {
            instance_store.get_instances()
        } else {
            self.instances.get_names()
        };

        let mut actions = Vec::new();
        for instance in &stop_instances {
            let mut action = StopInstanceAction::new(&instance_store.load(instance)?);
            action.run(instance_store, self.kill)?;
            actions.push(action);
        }

        if self.wait && !stop_instances.is_empty() {
            console.play(Arc::new(Mutex::new(Spinner::new(format!(
                "Stopping {}",
                stop_instances.join(", ")
            )))));
            while actions.iter().any(|action| !action.is_done(instance_store)) {
                thread::sleep(Duration::from_secs(1))
            }
            console.stop();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::Environment;
    use crate::platform::SystemMock;
    use std::rc::Rc;

    #[test]
    fn test_reject_path_traversal() {
        assert!(StopCommand::try_parse_from(["stop", "../../etc"]).is_err());
    }

    #[test]
    fn test_reject_empty_instance_list_without_all() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            "myuser".to_string(),
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
            StopCommand {
                all: false,
                wait: false,
                kill: false,
                instances: Vec::new().into(),
            }
            .run(console, &context),
            Err(Error::MissingInstanceName)
        ));
    }

    #[test]
    fn test_allow_empty_instance_list_with_all() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            "myuser".to_string(),
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

        assert!(
            StopCommand {
                all: true,
                wait: false,
                kill: false,
                instances: Vec::new().into(),
            }
            .run(console, &context)
            .is_ok()
        );
    }
}
