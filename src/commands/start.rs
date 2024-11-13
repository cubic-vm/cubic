use crate::error::Error;
use crate::machine::{MachineDao, MachineState};
use crate::view::InstanceStateChangeView;

pub fn start(
    machine_dao: &MachineDao,
    qemu_args: &Option<String>,
    console: bool,
    verbose: bool,
    ids: &Vec<String>,
) -> Result<(), Error> {
    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
    }

    let mut machines = Vec::new();
    for id in ids {
        let machine = machine_dao.load(id)?;
        machine_dao.start(&machine, qemu_args, console, verbose)?;
        machines.push(machine);
    }

    InstanceStateChangeView::new("Starting instance(s)", MachineState::Running)
        .run(machine_dao, &machines);

    Result::Ok(())
}
