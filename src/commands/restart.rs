use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::machine::MachineDao;

pub fn restart(
    machine_dao: &MachineDao,
    verbosity: Verbosity,
    instances: &Vec<String>,
) -> Result<(), Error> {
    commands::stop(machine_dao, false, verbosity, instances)?;
    commands::start(machine_dao, &None, verbosity, instances)
}
