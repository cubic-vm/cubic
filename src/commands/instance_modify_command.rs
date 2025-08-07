use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, PortForward};
use crate::util;
use clap::Parser;

/// Modify a virtual machine instance configuration
#[derive(Parser)]
pub struct InstanceModifyCommand {
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
    /// Add port forwarding rule (e.g. -p 8000:80)
    #[clap(short, long)]
    port: Vec<PortForward>,
    /// Remove port forwarding rule (e.g. -P 8000:80)
    #[clap(short = 'P', long)]
    rm_port: Vec<PortForward>,
}

impl InstanceModifyCommand {
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

        instance.hostfwd.append(&mut self.port.clone());
        instance.hostfwd.retain(|p| !self.rm_port.contains(p));

        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
