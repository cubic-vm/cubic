use crate::commands::image::ImageCommands;
use crate::error::Error;
use crate::image::{ImageDao, ImageStore};
use crate::instance::{Instance, InstanceDao, InstanceStore, USER};
use crate::util;
use clap::Parser;

const DEFAULT_CPU_COUNT: u16 = 4;
const DEFAULT_MEM_SIZE: &str = "4G";
const DEFAULT_DISK_SIZE: &str = "100G";

/// Create a new virtual machine instance
#[derive(Parser)]
pub struct InstanceAddCommand {
    /// Name of the virtual machine instance
    #[clap(conflicts_with = "name")]
    instance_name: Option<String>,
    /// Name of the virtual machine image
    #[clap(short, long)]
    image: String,
    /// Name of the virtual machine instance
    #[clap(short, long, conflicts_with = "instance_name", hide = true)]
    name: Option<String>,
    /// Number of CPUs for the virtual machine instance
    #[clap(short, long)]
    cpus: Option<u16>,
    /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
    #[clap(short, long)]
    mem: Option<String>,
    /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
    #[clap(short, long)]
    disk: Option<String>,
}

impl InstanceAddCommand {
    pub fn new(
        instance_name: String,
        image: String,
        cpus: Option<u16>,
        mem: Option<String>,
        disk: Option<String>,
    ) -> Self {
        Self {
            instance_name: Some(instance_name),
            image,
            name: None,
            cpus,
            mem,
            disk,
        }
    }

    pub fn run(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        let name = self
            .instance_name
            .as_ref()
            .or(self.name.as_ref())
            .ok_or(Error::InvalidArgument("Missing instance name".to_string()))?
            .to_string();

        if instance_dao.exists(&name) {
            return Result::Err(Error::InstanceAlreadyExists(name.to_string()));
        }

        ImageCommands::Fetch {
            image: self.image.to_string(),
        }
        .dispatch(image_dao)?;
        let image = image_dao.get(&self.image)?;
        image_dao.copy_image(&image, &name)?;

        let disk_capacity =
            util::human_readable_to_bytes(self.disk.as_deref().unwrap_or(DEFAULT_DISK_SIZE))?;
        let ssh_port = util::generate_random_ssh_port();

        let mut instance = Instance {
            name: name.clone(),
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
