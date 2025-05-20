use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, MountPoint};
use crate::util;
use crate::view::{Alignment, TableView};
use clap::Subcommand;
use std::path::Path;

#[derive(Subcommand)]
pub enum MountCommands {
    /// List mount mounts
    #[clap(alias = "ls")]
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
    #[clap(alias = "rm")]
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
                let mut view = TableView::new();
                view.add_row()
                    .add("Host", Alignment::Left)
                    .add("Guest", Alignment::Left);

                for mount in instance.mounts {
                    view.add_row()
                        .add(&mount.host, Alignment::Left)
                        .add(&mount.guest, Alignment::Left);
                }
                view.print();
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
