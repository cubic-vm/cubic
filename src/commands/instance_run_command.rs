use crate::commands;
use crate::commands::instance_add_command::InstanceAddCommand;
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use clap::Parser;

/// Create, start and open a shell in a new virtual machine instance
#[derive(Parser)]
pub struct InstanceRunCommand {
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
            .ok_or(Error::InvalidArgument("Missing instance name".to_string()))?
            .to_string();

        InstanceAddCommand::new(
            instance_name.clone(),
            self.image.to_string(),
            self.cpus.as_ref().cloned(),
            self.mem.as_ref().cloned(),
            self.disk.as_ref().cloned(),
        )
        .run(image_dao, instance_dao)?;
        commands::InstanceSshCommand {
            instance: instance_name.clone(),
            xforward: false,
            verbose: self.verbose,
            quiet: self.quiet,
            ssh_args: None,
            cmd: None,
        }
        .run(instance_dao)
    }
}
