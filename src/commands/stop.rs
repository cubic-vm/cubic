use crate::commands::Verbosity;
use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::view::TimerView;

pub fn stop(
    machine_dao: &MachineDao,
    all: bool,
    verbosity: Verbosity,
    instances: &Vec<String>,
) -> Result<(), Error> {
    for instance in instances {
        if !machine_dao.exists(instance) {
            return Result::Err(Error::UnknownMachine(instance.clone()));
        }
    }

    let stop_instances = if all {
        machine_dao.get_machines()
    } else {
        instances.clone()
    };

    let mut machines = Vec::new();
    for instance in stop_instances {
        let machine = machine_dao.load(&instance)?;
        machine_dao.stop(&machine)?;
        machines.push(machine);
    }

    if !verbosity.is_quiet() {
        TimerView::new("Stopping instance(s)").run(&mut || {
            machines
                .iter()
                .all(|machine| machine_dao.get_state(machine) == MachineState::Stopped)
        });
    }

    Result::Ok(())
}
