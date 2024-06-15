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

pub fn list_images(image_dao: &ImageDao) -> Result<(), Error> {
    println!("{:20} {: >5} {: >9}", "ID", "ARCH", "SIZE");
    for image in image_dao.get_images() {
        if !image_dao.exists(image) {
            continue;
        }

        let size = image_dao
            .get_capacity(image)
            .map(util::bytes_to_human_readable)
            .unwrap_or("n/a".to_string());
        println!(
            "{:20} {: >5} {: >9}",
            format!("{}:{}", image.vendor, image.codename),
            "amd64",
            size
        )
    }

    Result::Ok(())
}

pub fn list(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    name: &Option<String>,
) -> Result<(), Error> {
    match name.as_deref() {
        None | Some("machines") => list_machines(machine_dao),
        Some("images") => list_images(image_dao),
        Some(option) => Result::Err(Error::InvalidOption(option.to_string())),
    }
}
