use crate::commands::{
    InstanceAddCommand, InstanceCloneCommand, InstanceConfigCommand, InstanceListCommand,
    InstanceRemoveCommand, InstanceRenameCommand, Verbosity,
};
use crate::error::Error;
use crate::image::ImageDao;
use crate::instance::InstanceDao;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum InstanceCommands {
    /// List instances (Deprecated)
    #[clap(alias = "list")]
    Ls,

    /// Add a virtual machine instance (Deprecated)
    Add {
        /// Name of the virtual machine image
        #[clap(short, long)]
        image: String,
        /// Name of the virtual machine instance
        #[clap(short, long)]
        name: String,
        /// Number of CPUs for the virtual machine instance
        #[clap(short, long)]
        cpus: Option<u16>,
        /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
        #[clap(short, long)]
        mem: Option<String>,
        /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
        #[clap(short, long)]
        disk: Option<String>,
    },

    /// Delete instances (Deprecated)
    #[clap(alias = "del")]
    Rm {
        /// Enable verbose logging
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
        /// Reduce logging output
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// Delete the virtual machine instances without confirmation
        #[clap(short, long, default_value_t = false)]
        force: bool,
        /// Name of the virtual machine instances to delete
        instances: Vec<String>,
    },

    /// Read and write configuration parameters (Deprecated)
    Config {
        /// Name of the virtual machine instance
        instance: String,
        /// Number of CPUs for the virtual machine instance
        #[clap(short, long)]
        cpus: Option<u16>,
        /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
        #[clap(short, long)]
        mem: Option<String>,
        /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
        #[clap(short, long)]
        disk: Option<String>,
    },

    /// Clone a virtual machine instance (Deprecated)
    Clone {
        /// Name of the virtual machine instance to clone
        name: String,
        /// Name of the copy
        new_name: String,
    },

    /// Rename an instance (Deprecated)
    Rename {
        /// Name of the virtual machine instance to rename
        old_name: String,
        /// New name of the virutal machine instance
        new_name: String,
    },
}

impl InstanceCommands {
    pub fn dispatch(&self, image_dao: &ImageDao, instance_dao: &InstanceDao) -> Result<(), Error> {
        match self {
            InstanceCommands::Ls => InstanceListCommand::new().run(instance_dao),
            InstanceCommands::Add {
                image,
                name,
                cpus,
                mem,
                disk,
            } => InstanceAddCommand::new(
                image.to_string(),
                name.to_string(),
                cpus.as_ref().cloned(),
                mem.as_ref().cloned(),
                disk.as_ref().cloned(),
            )
            .run(image_dao, instance_dao),
            InstanceCommands::Rm {
                verbose,
                quiet,
                force,
                instances,
            } => InstanceRemoveCommand::new(Verbosity::new(*verbose, *quiet), *force, instances)
                .run(instance_dao),
            InstanceCommands::Config {
                instance,
                cpus,
                mem,
                disk,
            } => InstanceConfigCommand::new(instance, cpus, mem, disk).run(instance_dao),

            InstanceCommands::Clone { name, new_name } => {
                InstanceCloneCommand::new(name, new_name).run(instance_dao)
            }

            InstanceCommands::Rename { old_name, new_name } => {
                InstanceRenameCommand::new(old_name, new_name).run(instance_dao)
            }
        }
    }
}
