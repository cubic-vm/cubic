use crate::commands::Verbosity;
use crate::error::Error;
use crate::instance::{InstanceDao, InstanceState};
use crate::view::TimerView;

pub fn stop(
    instance_dao: &InstanceDao,
    all: bool,
    verbosity: Verbosity,
    instances: &Vec<String>,
) -> Result<(), Error> {
    for instance in instances {
        if !instance_dao.exists(instance) {
            return Result::Err(Error::UnknownInstance(instance.clone()));
        }
    }

    let stop_instances = if all {
        instance_dao.get_instances()
    } else {
        instances.clone()
    };

    let mut instances = Vec::new();
    for instance in stop_instances {
        let instance = instance_dao.load(&instance)?;
        instance_dao.stop(&instance)?;
        instances.push(instance);
    }

    if !verbosity.is_quiet() {
        TimerView::new("Stopping instance(s)").run(&mut || {
            instances
                .iter()
                .all(|instance| instance_dao.get_state(instance) == InstanceState::Stopped)
        });
    }

    Result::Ok(())
}
