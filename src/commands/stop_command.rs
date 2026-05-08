use crate::actions::StopInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::view::Console;
use crate::view::SpinnerView;
use clap::Parser;
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
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct StopCommand {
    /// Stop all virtual machine instances
    #[clap(short, long, default_value_t = false)]
    pub all: bool,
    /// Wait for the virtual machine instance to be stopped
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Name of the virtual machine instances to stop
    pub instances: Vec<String>,
}

impl Command for StopCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        let stop_instances = if self.all {
            instance_store.get_instances()
        } else {
            self.instances.clone()
        };

        let mut actions = Vec::new();
        for instance in &stop_instances {
            let mut action = StopInstanceAction::new(&instance_store.load(instance)?);
            action.run(instance_store)?;
            actions.push(action);
        }

        if self.wait && !console.get_verbosity().is_quiet() {
            let mut spinner = SpinnerView::new("Stopping instance(s)".to_string());
            while actions.iter().any(|action| !action.is_done(instance_store)) {
                thread::sleep(Duration::from_secs(1))
            }
            spinner.stop();
        }

        Ok(())
    }
}
