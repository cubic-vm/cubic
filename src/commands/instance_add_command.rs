use crate::commands::image::ImageCommands;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{Instance, InstanceDao, InstanceStore, USER};
use crate::util;

const DEFAULT_CPU_COUNT: u16 = 4;
const DEFAULT_MEM_SIZE: &str = "4G";
const DEFAULT_DISK_SIZE: &str = "100G";

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
        if instance_dao.exists(&self.name) {
            return Result::Err(Error::InstanceAlreadyExists(self.name.to_string()));
        }

        ImageCommands::Fetch {
            image: self.image.to_string(),
        }
        .dispatch(image_dao)?;
        let image = image_dao.get(&self.image)?;
        image_dao.copy_image(&image, &self.name)?;

        let disk_capacity =
            util::human_readable_to_bytes(self.disk.as_deref().unwrap_or(DEFAULT_DISK_SIZE))?;
        let ssh_port = util::generate_random_ssh_port();

        let mut instance = Instance {
            name: self.name.clone(),
            arch: image.arch,
            user: USER.to_string(),
            cpus: self.cpus.unwrap_or(DEFAULT_CPU_COUNT),
            mem: util::human_readable_to_bytes(self.mem.as_deref().unwrap_or(DEFAULT_MEM_SIZE))?,
            disk_capacity: 0, // Will be overwritten by resize operation below
            ssh_port,
            ..Instance::default()
        };
        instance_dao.resize(&mut instance, disk_capacity)?;
        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
