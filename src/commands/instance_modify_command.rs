use crate::error::Error;
use crate::instance::{InstanceDao, InstanceStore, PortForward};
use crate::model::DataSize;
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
    mem: Option<DataSize>,
    /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
    #[clap(short, long)]
    disk: Option<DataSize>,
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
            instance.mem = mem.get_bytes() as u64;
        }

        if let Some(disk) = &self.disk {
            instance_dao.resize(&mut instance, disk.get_bytes() as u64)?;
        }

        instance.hostfwd.append(&mut self.port.clone());
        instance.hostfwd.retain(|p| !self.rm_port.contains(p));

        instance_dao.store(&instance)?;
        Result::Ok(())
    }
}
