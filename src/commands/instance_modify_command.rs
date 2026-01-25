use crate::commands::Command;
use crate::env::Environment;
use crate::error::Error;
use crate::image::ImageStore;
use crate::instance::{InstanceStore, PortForward};
use crate::model::DataSize;
use crate::view::Console;
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

impl Command for InstanceModifyCommand {
    fn run(
        &self,
        _console: &mut dyn Console,
        _env: &Environment,
        _image_store: &dyn ImageStore,
        instance_store: &dyn InstanceStore,
    ) -> Result<(), Error> {
        let mut instance = instance_store.load(&self.instance)?;

        if let Some(cpus) = &self.cpus {
            instance.cpus = *cpus;
        }

        if let Some(mem) = &self.mem {
            instance.mem = mem.clone();
        }

        if let Some(disk) = &self.disk {
            instance_store.resize(&mut instance, disk.get_bytes() as u64)?;
        }

        instance.hostfwd.append(&mut self.port.clone());
        instance.hostfwd.retain(|p| !self.rm_port.contains(p));

        instance_store.store(&instance)?;
        Result::Ok(())
    }
}
