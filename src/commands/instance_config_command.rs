use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore};
use crate::util;
use clap::Parser;

/// Modify virtual machine instance configuration
#[derive(Parser)]
pub struct InstanceConfigCommand {
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
}

impl InstanceConfigCommand {
    pub fn run(&self, instance_dao: &InstanceDao) -> Result<(), Error> {
        let mut instance = instance_dao.load(&self.instance)?;

        if let Some(cpus) = &self.cpus {
            instance.cpus = *cpus;
        }

        if let Some(mem) = &self.mem {
            instance.mem = util::human_readable_to_bytes(mem)?;
        }

        if let Some(disk) = &self.disk {
            instance_dao.resize(&mut instance, util::human_readable_to_bytes(disk)?)?;
        }

        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
