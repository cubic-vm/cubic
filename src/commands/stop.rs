use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::view::InstanceStateChangeView;

pub fn stop(machine_dao: &MachineDao, ids: &Vec<String>, all: bool) -> Result<(), Error> {
    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
    }

    let stop_ids = if all {
        machine_dao.get_machines()
    } else {
        ids.clone()
    };

    let mut machines = Vec::new();
    for id in stop_ids {
        let machine = machine_dao.load(&id)?;
        machine_dao.stop(&machine)?;
        machines.push(machine);
    }

    InstanceStateChangeView::new("Stopping instance(s)", MachineState::Stopped)
        .run(machine_dao, &machines);

    Result::Ok(())
}
