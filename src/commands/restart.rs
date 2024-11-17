use crate::commands;
use crate::error::Error;
use crate::machine::MachineDao;

pub fn restart(
    machine_dao: &MachineDao,
    console: bool,
    verbose: bool,
    quiet: bool,
    ids: &Vec<String>,
) -> Result<(), Error> {
    commands::stop(machine_dao, ids, false, quiet)?;
    commands::start(machine_dao, &None, console, verbose, quiet, ids)
}
