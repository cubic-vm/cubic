use crate::actions::StartInstanceAction;
use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::SpinnerView;
use clap::Parser;
use std::thread::sleep;
use std::time::Duration;

/// Start virtual machine instances
#[derive(Parser)]
pub struct InstanceStartCommand {
    /// Pass additional QEMU arguments
    #[clap(long)]
    pub qemu_args: Option<String>,
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    pub quiet: bool,
    /// Wait for the virtual machine instance to be started
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Name of the virtual machine instances to start
    pub instances: Vec<String>,
}

impl InstanceStartCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let verbosity = Verbosity::new(self.verbose, self.quiet);
        // Launch virtual machine instances
        let mut actions = Vec::new();
        for name in &self.instances {
            let instance = &instance_dao.load(name)?;
            if !instance_dao.is_running(instance) {
                let mut action = StartInstanceAction::new(instance);
                action.run(
                    instance_dao,
                    &instance_dao.env,
                    &self.qemu_args,
                    verbosity.is_verbose(),
                )?;

                actions.push(action);
            }
        }

        // Wait for virtual machine instances to be started
        if self.wait && !verbosity.is_quiet() {
            let mut spinner = SpinnerView::new("Starting instance(s)");
            while actions.iter().any(|a| !a.is_done()) {
                sleep(Duration::from_secs(1));
            }
            spinner.stop()
        }

        Result::Ok(())
    }
}
