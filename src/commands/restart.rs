use crate::error::Error;
use crate::machine::MachineDao;
use std::thread;
use std::time::Duration;

pub fn restart(machine_dao: &MachineDao, console: bool, ids: &Vec<String>) -> Result<(), Error> {
    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
    }

    for id in ids {
        let machine = machine_dao.load(id)?;
        machine_dao.stop(&machine)?;
        thread::sleep(Duration::new(2, 0));
        machine_dao.start(&machine, &None, console)?;
    }

    Result::Ok(())
}
