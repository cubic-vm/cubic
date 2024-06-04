use crate::error::Error;
use crate::image::{ImageDao, ImageName};
use crate::machine::{Machine, MachineDao};
use crate::util::{self, copy_file, create_dir, generate_random_ssh_port};

pub fn add(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    image_name: &str,
    name: &Option<String>,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
) -> Result<(), Error> {
    let image_name = ImageName::from_id(image_name)?;
    image_dao.add(&image_name)?;

    if let Option::Some(instance) = name {
        let machine_dir = format!("{}/{instance}", machine_dao.machine_dir);
        let machine_image = format!("{machine_dir}/machine.img");

        if let Some(id) = name {
            if machine_dao.exists(id) {
                return Result::Err(Error::MachineAlreadyExists(id.to_string()));
            }
        }

        let image = image_dao.load(&image_name)?;
        let disk_capacity = disk
            .as_ref()
            .map(|size| util::human_readable_to_bytes(size))
            .unwrap_or(Result::Ok(image.size))?;

        create_dir(&machine_dir)?;
        copy_file(&image.path, &machine_image)?;

        let ssh_port = generate_random_ssh_port();

        let mut machine = Machine {
            name: instance.clone(),
            cpus: cpus.unwrap_or(1),
            mem: util::human_readable_to_bytes(mem.as_deref().unwrap_or("1G"))?,
            disk_capacity,
            ssh_port,
            sandbox: false,
        };
        machine_dao.store(&machine)?;
        if disk.is_some() {
            machine_dao.resize(&mut machine, disk_capacity)?;
        }
    }

    Result::Ok(())
}
