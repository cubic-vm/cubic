use crate::error::Error;
use crate::machine::{MachineDao, MountPoint};
use crate::util;
use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum MountCommands {
    /// List mount mounts
    List { name: String },

    /// Add a directory mount
    Add {
        name: String,
        host: String,
        guest: String,
    },

    /// Delete a directory mount
    Del { name: String, guest: String },
}

impl MountCommands {
    pub fn dispatch(&self, machine_dao: &MachineDao) -> Result<(), Error> {
        match self {
            MountCommands::List { name } => {
                let machine = machine_dao.load(name)?;
                println!("{:30} {:30}", "HOST", "GUEST");
                for mount in machine.mounts {
                    println!("{:30} {:30}", mount.host, mount.guest);
                }
                Ok(())
            }

            MountCommands::Add { name, host, guest } => {
                if !Path::new(host).exists() {
                    return Err(Error::CannotAccessDir(host.clone()));
                }

                let mut machine = machine_dao.load(name)?;
                machine.mounts.push(MountPoint {
                    host: host.to_string(),
                    guest: guest.to_string(),
                });
                machine_dao.store(&machine)?;
                util::setup_cloud_init(&machine, &machine_dao.cache_dir, true)
            }

            MountCommands::Del {
                ref name,
                ref guest,
            } => {
                let mut machine = machine_dao.load(name)?;
                machine.mounts.retain(|mount| mount.guest != *guest);
                machine_dao.store(&machine)?;
                util::setup_cloud_init(&machine, &machine_dao.cache_dir, true)
            }
        }
    }
}
