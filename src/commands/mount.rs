use crate::error::Error;
use crate::instance::{InstanceDao, MountPoint};
use crate::util;
use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum MountCommands {
    /// List mount mounts
    List {
        /// Name of the virtual machine instance
        name: String,
    },

    /// Add a directory mount
    Add {
        /// Name of the virtual machine instance
        name: String,
        /// Path on the host filesystem
        host: String,
        /// Path on guest filesystem
        guest: String,
    },

    /// Delete a directory mount
    Del {
        /// Name of the virtual machine instance
        name: String,
        /// Path on guest filesystem
        guest: String,
    },
}

impl MountCommands {
    pub fn dispatch(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        match self {
            MountCommands::List { name } => {
                let instance = instance_dao.load(name)?;
                println!("{:30} {:30}", "HOST", "GUEST");
                for mount in instance.mounts {
                    println!("{:30} {:30}", mount.host, mount.guest);
                }
                Ok(())
            }

            MountCommands::Add { name, host, guest } => {
                if !Path::new(host).exists() {
                    return Err(Error::CannotAccessDir(host.clone()));
                }

                let mut instance = instance_dao.load(name)?;
                instance.mounts.push(MountPoint {
                    host: host.to_string(),
                    guest: guest.to_string(),
                });
                instance_dao.store(&instance)?;
                util::setup_cloud_init(&instance, &instance_dao.cache_dir, true)
            }

            MountCommands::Del {
                ref name,
                ref guest,
            } => {
                let mut instance = instance_dao.load(name)?;
                instance.mounts.retain(|mount| mount.guest != *guest);
                instance_dao.store(&instance)?;
                util::setup_cloud_init(&instance, &instance_dao.cache_dir, true)
            }
        }
    }
}
