use crate::commands;
use crate::commands::instance_add_command::{
    InstanceAddCommand, DEFAULT_CPU_COUNT, DEFAULT_DISK_SIZE, DEFAULT_MEM_SIZE,
};
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::{InstanceDao, InstanceName, PortForward, Target};
use clap::Parser;

/// Create, start and open a shell in a new virtual machine instance
#[derive(Parser)]
pub struct InstanceRunCommand {
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
    /// Enable verbose logging
    #[clap(short, long, default_value_t = false)]
    verbose: bool,
    /// Reduce logging output
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
}

impl InstanceRunCommand {
    pub fn run(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        let instance_name = self
            .instance_name
            .as_ref()
            .or(self.name.as_ref())
            .ok_or(Error::InvalidArgument("Missing instance name".to_string()))?;

        InstanceAddCommand::new(
            instance_name.clone(),
            self.image.to_string(),
            self.user.clone(),
            self.cpus,
            self.mem.clone(),
            self.disk.clone(),
            self.port.clone(),
        )
        .run(image_dao, instance_dao)?;
        commands::InstanceSshCommand {
            target: Target::from_instance_name(instance_name.clone()),
            xforward: false,
            verbose: self.verbose,
            quiet: self.quiet,
            ssh_args: None,
            cmd: None,
        }
        .run(instance_dao)
    }
}
