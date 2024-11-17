use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::InstanceDao;

pub fn restart(
    instance_dao: &InstanceDao,
    verbosity: Verbosity,
    instances: &Vec<String>,
) -> Result<(), Error> {
    commands::stop(instance_dao, false, verbosity, instances)?;
    commands::start(instance_dao, &None, verbosity, instances)
}
