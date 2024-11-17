use crate::commands::{self, Verbosity};
use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::MachineDao;

#[allow(clippy::too_many_arguments)]
pub fn run(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    image_name: &str,
    name: &String,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
    verbosity: Verbosity,
) -> Result<(), Error> {
    commands::InstanceCommands::add_instance(
        image_dao,
        machine_dao,
        image_name,
        &Some(name.to_string()),
        cpus,
        mem,
        disk,
    )?;
    commands::ssh(machine_dao, name, false, verbosity, &None, &None)
}
