use crate::error::Error;
use crate::image::ImageDao;
use crate::machine::{Machine, MachineDao};
use crate::util::{self, generate_random_ssh_port};

pub fn add(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    image_name: &str,
    name: &Option<String>,
    cpus: &Option<u16>,
    mem: &Option<String>,
    disk: &Option<String>,
) -> Result<(), Error> {
    let image = image_dao.get(image_name)?;
    image_dao.fetch(&image)?;

    if let Option::Some(instance) = name {
        let machine_dir = format!("{}/{instance}", machine_dao.machine_dir);

        if let Some(id) = name {
            if machine_dao.exists(id) {
                return Result::Err(Error::MachineAlreadyExists(id.to_string()));
            }
        }

        let image_size = image_dao.get_disk_capacity(&image)?;
        let disk_capacity = disk
            .as_ref()
            .map(|size| util::human_readable_to_bytes(size))
            .unwrap_or(Result::Ok(image_size))?;

        image_dao.copy_image(&image, &machine_dir, "machine.img")?;

        let ssh_port = generate_random_ssh_port();

        let mut machine = Machine {
            name: instance.clone(),
            cpus: cpus.unwrap_or(1),
            mem: util::human_readable_to_bytes(mem.as_deref().unwrap_or("1G"))?,
            disk_capacity,
            ssh_port,
            sandbox: false,
            mounts: Vec::new(),
        };
        machine_dao.store(&machine)?;
        if disk.is_some() {
            machine_dao.resize(&mut machine, disk_capacity)?;
        }
    }

    Result::Ok(())
}
