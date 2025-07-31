use crate::actions::StartInstanceAction;
use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::SpinnerView;
use std::thread::sleep;
use std::time::Duration;

pub fn start(
    instance_dao: &InstanceDao,
    qemu_args: &Option<String>,
    verbosity: Verbosity,
    wait: bool,
    instance_names: &Vec<String>,
) -> Result<(), Error> {
    // Launch virtual machine instances
    let mut actions = Vec::new();
    for name in instance_names {
        let instance = &instance_dao.load(name)?;
        if !instance_dao.is_running(instance) {
            let mut action = StartInstanceAction::new(instance);
            action.run(
                instance_dao,
                &instance_dao.env,
                qemu_args,
                verbosity.is_verbose(),
            )?;

            actions.push(action);
        }
    }

    // Wait for virtual machine instances to be started
    if wait && !verbosity.is_quiet() {
        let mut spinner = SpinnerView::new("Starting instance(s)");
        while actions.iter().any(|a| !a.is_done()) {
            sleep(Duration::from_secs(1));
        }
        spinner.stop()
    }

    Result::Ok(())
}
