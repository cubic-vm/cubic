use crate::actions::StopInstanceAction;
use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::view::SpinnerView;
use std::thread;
use std::time::Duration;

pub fn stop(
    instance_dao: &InstanceDao,
    all: bool,
    verbosity: Verbosity,
    instances: &[String],
) -> Result<(), Error> {
    let stop_instances = if all {
        instance_dao.get_instances()
    } else {
        instances.to_vec()
    };

    let mut actions = Vec::new();
    for instance in stop_instances {
        let mut action = StopInstanceAction::new(&instance_dao.load(&instance)?);
        action.run(instance_dao)?;
        actions.push(action);
    }

    if !verbosity.is_quiet() {
        let mut spinner = SpinnerView::new("Stopping instance(s)");
        while actions.iter().any(|action| !action.is_done(instance_dao)) {
            thread::sleep(Duration::from_secs(1))
        }
        spinner.stop();
    }

    Ok(())
}
