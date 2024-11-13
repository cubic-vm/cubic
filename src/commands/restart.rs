use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::view::InstanceStateChangeView;

pub fn restart(
    machine_dao: &MachineDao,
    console: bool,
    verbose: bool,
    ids: &Vec<String>,
) -> Result<(), Error> {
    let mut machines = Vec::new();
    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
        let machine = machine_dao.load(id)?;
        machines.push(machine);
    }

    for id in ids {
        let machine = machine_dao.load(id)?;
        machine_dao.stop(&machine)?;
        InstanceStateChangeView::new("Stopping instance(s)", MachineState::Stopped)
            .run(machine_dao, &machines);

        machine_dao.start(&machine, &None, console, verbose)?;
        InstanceStateChangeView::new("Starting instance(s)", MachineState::Running)
            .run(machine_dao, &machines);
    }

    Result::Ok(())
}
