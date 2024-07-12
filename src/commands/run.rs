use crate::commands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::MachineDao;

pub fn run(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    image_name: &str,
    name: &String,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
) -> Result<(), Error> {
    commands::add(
        image_dao,
        machine_dao,
        image_name,
        &Some(name.to_string()),
        cpus,
        mem,
        disk,
    )?;
    commands::ssh(machine_dao, name, &None, &None)
}
