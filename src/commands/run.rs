use crate::commands::instance_add_command::InstanceAddCommand;
use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;

#[allow(clippy::too_many_arguments)]
pub fn run(
    image_dao: &ImageDao,
    instance_dao: &InstanceDao,
    image_name: &str,
    name: &String,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
    verbosity: Verbosity,
) -> Result<(), Error> {
    InstanceAddCommand::new(
        image_name.to_string(),
        name.to_string(),
        cpus.as_ref().cloned(),
        mem.as_ref().cloned(),
        disk.as_ref().cloned(),
    )
    .run(image_dao, instance_dao)?;
    commands::sh(instance_dao, verbosity, name)
}
