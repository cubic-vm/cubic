use crate::error::Error;
use crate::machine::MachineDao;
use crate::util;

pub fn start(machine_dao: &MachineDao, console: bool, ids: &Vec<String>) -> Result<(), Error> {
    util::check_ssh_key();

    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
    }

    for id in ids {
        let machine = machine_dao.load(id)?;
        machine_dao.start(&machine, console)?;
    }

    Result::Ok(())
}
