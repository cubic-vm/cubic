use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::MachineDao;
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
            if machine_dao.is_running(&machine) {
                "RUNNING"
            } else {
                "STOPPED"
            }
        );
    }

    Result::Ok(())
}

pub fn list_images(image_dao: &ImageDao) -> Result<(), Error> {
    println!("{:20} {: >5} {: >9}", "ID", "ARCH", "SIZE");
    for name in image_dao.get_images() {
        let image = image_dao.load(&name)?;
        println!(
            "{:20} {: >5} {: >9}",
            format!("{}:{}", name.vendor, name.image),
            name.arch,
            util::bytes_to_human_readable(image.size)
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
