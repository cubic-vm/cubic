use crate::actions::StopInstanceAction;
use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::view::Console;
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

impl Command for InstanceStopCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
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
            let mut spinner = SpinnerView::new("Stopping instance(s)");
            while actions.iter().any(|action| !action.is_done(instance_store)) {
                thread::sleep(Duration::from_secs(1))
            }
            spinner.stop();
        }

        Ok(())
    }
}
