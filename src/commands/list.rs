use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::{MachineDao, MachineState};
use crate::util;

pub fn list_machines(machine_dao: &MachineDao) -> Result<(), Error> {
    let machine_names = machine_dao.get_machines();

    println!(
        "{:15}  {: >4}  {: >9}  {: >9}  {:10}",
        "Name", "CPUs", "Memory", "Disk", "State"
    );
    for machine_name in machine_names {
        let machine = machine_dao.load(&machine_name)?;
        println!(
            "{:15}  {: >4}  {: >9}  {: >9}  {:10}",
            machine_name,
            &machine.cpus,
            util::bytes_to_human_readable(machine.mem),
            util::bytes_to_human_readable(machine.disk_capacity),
            match machine_dao.get_state(&machine) {
                MachineState::Stopped => "STOPPED",
                MachineState::Starting => "STARTING",
                MachineState::Running => "RUNNING",
            }
        );
    }

    Result::Ok(())
}

pub fn list_images(image_dao: &ImageDao, all: bool) -> Result<(), Error> {
    println!(
        "{:6}  {:>7}  {:10}  {: >5}  {: >9}",
        "Vendor", "Version", "Name", "Arch", "Size"
    );
    for image in image_dao.get_images() {
        if !(all || image_dao.exists(image)) {
            continue;
        }

        let size = image_dao
            .get_disk_size(image)
            .map(util::bytes_to_human_readable)
            .unwrap_or_default();
        println!(
            "{:6}  {:>7}  {:10}  {: >5}  {: >9}",
            image.vendor, image.version, image.codename, "amd64", size
        )
    }

    Result::Ok(())
}

pub fn list(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    name: &Option<String>,
    all: bool,
) -> Result<(), Error> {
    match name.as_deref() {
        None | Some("machines") => list_machines(machine_dao),
        Some("images") => list_images(image_dao, all),
        Some(option) => Result::Err(Error::InvalidOption(option.to_string())),
    }
}
