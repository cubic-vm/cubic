use crate::actions::CreateInstanceAction;
use crate::commands::Command;
use crate::commands::image::ImageCommands;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{ImageName, ImageStore};
use crate::instance::{Instance, InstanceName, InstanceStore, PortForward};
use crate::model::DataSize;
use crate::ssh_cmd::PortChecker;
use crate::view::Console;
use crate::view::SpinnerView;
use clap::Parser;

pub const DEFAULT_CPU_COUNT: u16 = 4;
pub const DEFAULT_MEM_SIZE: &str = "4G";
pub const DEFAULT_DISK_SIZE: &str = "100G";

/// Create a new virtual machine instance
#[derive(Parser)]
pub struct CreateInstanceCommand {
    /// Name of the virtual machine instance
    #[clap(conflicts_with = "name")]
    instance_name: Option<InstanceName>,
    /// Name of the virtual machine image
    #[clap(short, long)]
    image: ImageName,
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
    mem: DataSize,
    /// Disk size of the virtual machine instance (e.g. 10G for 10 gigabytes)
    #[clap(short, long, default_value = DEFAULT_DISK_SIZE)]
    disk: DataSize,
    /// Forward ports from guest to host (e.g. -p 8000:80 or -p 127.0.0.1:9000:90/tcp)
    #[clap(short, long)]
    port: Vec<PortForward>,
}

impl CreateInstanceCommand {
    pub fn get_name(&self) -> Result<InstanceName, Error> {
        self.instance_name
            .clone()
            .or(self.name.clone())
            .ok_or(Error::InvalidArgument("Missing instance name".to_string()))
    }
}

impl Command for CreateInstanceCommand {
    fn run(
        &self,
        console: &mut dyn Console,
        env: &Environment,
        image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let name = self.get_name()?;

        if instance_store.exists(name.as_str()) {
            return Result::Err(Error::InstanceAlreadyExists(name.to_string()));
        }

        ImageCommands::Fetch {
            image: self.image.clone(),
        }
        .run(console, env, image_store, instance_store)?;

        let mut create_spinner = SpinnerView::new("Creating virtual machine instance");
        let image = image_store.get(&self.image)?;

        let instance = Instance {
            name: name.to_string(),
            arch: image.arch,
            user: self.user.to_string(),
            cpus: self.cpus,
            mem: self.mem.get_bytes() as u64,
            disk_capacity: self.disk.get_bytes() as u64,
            ssh_port: PortChecker::new().get_new_port(),
            hostfwd: self.port.clone(),
            ..Instance::default()
        };
        CreateInstanceAction::new().run(env, &FS::new(), instance_store, &image, instance)?;

        create_spinner.stop();
        Result::Ok(())
    }
}
