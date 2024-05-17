use crate::error::Error;
use crate::machine::MachineDao;

pub fn start(machine_dao: &MachineDao, ids: &Vec<String>) -> Result<(), Error> {
    for id in ids {
        if !machine_dao.exists(id) {
            return Result::Err(Error::UnknownMachine(id.clone()));
        }
    }

    for id in ids {
        let machine = machine_dao.load(id)?;
        machine_dao.start(&machine)?;
    }

    Result::Ok(())
}
