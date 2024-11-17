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
    commands::InstanceCommands::add_instance(
        image_dao,
        instance_dao,
        image_name,
        &Some(name.to_string()),
        cpus,
        mem,
        disk,
    )?;
    commands::ssh(instance_dao, name, false, verbosity, &None, &None)
}
