use crate::commands::image::ImageCommands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{Instance, InstanceDao, InstanceStore, USER};
use crate::util;

pub struct InstanceAddCommand {
    image: String,
    name: String,
    cpus: Option<u16>,
    mem: Option<String>,
    disk: Option<String>,
}

impl InstanceAddCommand {
    pub fn new(
        image: String,
        name: String,
        cpus: Option<u16>,
        mem: Option<String>,
        disk: Option<String>,
    ) -> Self {
        InstanceAddCommand {
            image,
            name,
            cpus,
            mem,
            disk,
        }
    }

    pub fn run(self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        let image = image_dao.get(&self.image)?;
        ImageCommands::Fetch {
            image: self.image.to_string(),
        }
        .dispatch(image_dao)?;

        let instance_dir = format!("{}/{}", instance_dao.instance_dir, &self.name);

        if instance_dao.exists(&self.name) {
            return Result::Err(Error::InstanceAlreadyExists(self.name.to_string()));
        }

        let image_size = image_dao.get_disk_capacity(&image)?;
        let disk_capacity = self
            .disk
            .as_ref()
            .map(|size| util::human_readable_to_bytes(size))
            .unwrap_or(Result::Ok(image_size))?;

        image_dao.copy_image(&image, &instance_dir, "machine.img")?;

        let ssh_port = util::generate_random_ssh_port();

        let mut instance = Instance {
            name: self.name.clone(),
            user: USER.to_string(),
            cpus: self.cpus.unwrap_or(1),
            mem: util::human_readable_to_bytes(self.mem.as_deref().unwrap_or("1G"))?,
            disk_capacity,
            ssh_port,
            ..Instance::default()
        };
        instance_dao.store(&instance)?;
        if self.disk.is_some() {
            instance_dao.resize(&mut instance, disk_capacity)?;
        }

        Result::Ok(())
    }
}
