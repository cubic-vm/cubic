use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::instance::InstanceDao;

pub fn restart(
    instance_dao: &InstanceDao,
    verbosity: Verbosity,
    instances: &[String],
) -> Result<(), Error> {
    commands::stop(instance_dao, false, verbosity, true, instances)?;
    commands::InstanceStartCommand {
        qemu_args: None,
        verbose: verbosity.is_verbose(),
        quiet: verbosity.is_quiet(),
        wait: true,
        instances: instances.to_vec(),
    }
    .run(instance_dao)
}
