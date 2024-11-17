use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::machine::MachineDao;

pub fn restart(
    machine_dao: &MachineDao,
    verbosity: Verbosity,
    ids: &Vec<String>,
) -> Result<(), Error> {
    commands::stop(machine_dao, ids, false, verbosity)?;
    commands::start(machine_dao, &None, verbosity, ids)
}
