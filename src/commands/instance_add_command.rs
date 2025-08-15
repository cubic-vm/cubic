use crate::commands::image::ImageCommands;
use crate::error::Error;
use crate::image::{ImageDao, ImageStore};
use crate::instance::{Instance, InstanceDao, InstanceName, InstanceStore, PortForward};
use crate::util;
use clap::Parser;

pub const DEFAULT_CPU_COUNT: u16 = 4;
pub const DEFAULT_MEM_SIZE: &str = "4G";
pub const DEFAULT_DISK_SIZE: &str = "100G";

/// Create a new virtual machine instance
#[derive(Parser)]
pub struct InstanceAddCommand {
    /// Name of the virtual machine instance
    #[clap(conflicts_with = "name")]
    instance_name: Option<InstanceName>,
    /// Name of the virtual machine image
    #[clap(short, long)]
    image: String,
    /// Name of the virtual machine instance
    #[clap(short, long, conflicts_with = "instance_name", hide = true)]
    name: Option<InstanceName>,
    /// Name of the user
    #[clap(short, long, default_value = "cubic")]
    user: String,
    /// Number of CPUs for the virtual machine instance
    #[clap(short, long, default_value_t = DEFAULT_CPU_COUNT)]
    cpus: u16,
    /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
    #[clap(short, long, default_value = DEFAULT_MEM_SIZE)]
    mem: String,
    /// Disk size of the virtual machine instance (e.g. 10G for 10 gigabytes)
    #[clap(short, long, default_value = DEFAULT_DISK_SIZE)]
    disk: String,
    /// Forward ports from guest to host (e.g. -p 8000:80 or -p 127.0.0.1:9000:90/tcp)
    #[clap(short, long)]
    port: Vec<PortForward>,
}

impl InstanceAddCommand {
    pub fn get_name(&self) -> Result<InstanceName, Error> {
        self.instance_name
            .clone()
            .or(self.name.clone())
            .ok_or(Error::InvalidArgument("Missing instance name".to_string()))
    }

    pub fn run(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        let name = self.get_name()?;

        if instance_dao.exists(name.as_str()) {
            return Result::Err(Error::InstanceAlreadyExists(name.to_string()));
        }

        ImageCommands::Fetch {
            image: self.image.to_string(),
        }
        .dispatch(image_dao)?;
        let image = image_dao.get(&self.image)?;
        image_dao.copy_image(&image, name.as_str())?;

        let disk_capacity = util::human_readable_to_bytes(&self.disk)?;
        let ssh_port = util::generate_random_ssh_port();

        let mut instance = Instance {
            name: name.to_string(),
            arch: image.arch,
            user: self.user.to_string(),
            cpus: self.cpus,
            mem: util::human_readable_to_bytes(&self.mem)?,
            disk_capacity: 0, // Will be overwritten by resize operation below
            ssh_port,
            hostfwd: self.port.clone(),
        };
        instance_dao.resize(&mut instance, disk_capacity)?;
        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
