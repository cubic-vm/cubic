use crate::actions::StopInstanceAction;
use crate::commands;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::SpinnerView;
use clap::Parser;
use std::thread;
use std::time::Duration;

/// Stop virtual machine instances
#[derive(Parser)]
pub struct InstanceStopCommand {
    /// Stop all virtual machine instances
    #[clap(short, long, default_value_t = false)]
    pub all: bool,
    /// Wait for the virtual machine instance to be stopped
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Name of the virtual machine instances to stop
    pub instances: Vec<String>,
}

impl InstanceStopCommand {
    pub fn run(
        &self,
        instance_dao: &InstanceDao,
        verbosity: commands::Verbosity,
    ) -> Result<(), Error> {
        let stop_instances = if self.all {
            instance_dao.get_instances()
        } else {
            self.instances.clone()
        };

        let mut actions = Vec::new();
        for instance in &stop_instances {
            let mut action = StopInstanceAction::new(&instance_dao.load(instance)?);
            action.run(instance_dao)?;
            actions.push(action);
        }

        if self.wait && !verbosity.is_quiet() {
            let mut spinner = SpinnerView::new("Stopping instance(s)");
            while actions.iter().any(|action| !action.is_done(instance_dao)) {
                thread::sleep(Duration::from_secs(1))
            }
            spinner.stop();
        }

        Ok(())
    }
}
