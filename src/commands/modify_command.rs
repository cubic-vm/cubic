use crate::commands::{self, Command};
use crate::error::Result;
use crate::instance::PortForward;
use crate::model::DataSize;
use crate::view::Console;
use clap::{ArgAction, Parser};

/// Modify VM instances
///
/// Use this command to change the settings of an existing VM instance (CPU, memory,
/// disk, etc.). The changes will be applied on the next (re-)start of the VM
/// instance.
///
/// Examples:
///
///   Assign 8 virtual CPUs to a VM instance:
///   $ cubic modify example1 --cpus 8
///
///   Assign 10 GiB of RAM to a VM instance:
///   $ cubic modify example2 --memory 10G
///
///   Assign 200 GiB of storage to a VM instance:
///   $ cubic modify example3 --disk 200G
///
///   Forward the VM instance's SSH port (TCP port 22) to the host on port 2222:
///   $ cubic modify example4 --port 2222:22
///
///   Forward the VM instance's DNS port (UDP port 53) to the host on port 5353:
///   $ cubic modify example5 --port 127.0.0.1:5353:53/udp
///
///   Remove DNS port forwarding rule:
///   $ cubic modify example6 --rm-port 127.0.0.1:5353:53/udp
///
///   Deny network access (host, LAN, internet, ...) of a VM instance:
///   $ cubic modify example7 --isolate
///
///   Allow network connection of a VM instance:
///   $ cubic modify example8 --no-isolate
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct ModifyCommand {
    /// Name of the virtual machine instance
    instance: String,
    /// Number of CPUs for the virtual machine instance
    #[clap(short, long)]
    cpus: Option<u16>,
    /// Memory size of the virtual machine instance (e.g. 1G for 1 gigabyte)
    #[clap(alias = "mem", short, long)]
    memory: Option<DataSize>,
    /// Disk size of the virtual machine instance  (e.g. 10G for 10 gigabytes)
    #[clap(short, long)]
    disk: Option<DataSize>,
    /// Add port forwarding rule (format: [host_ip:]host_port:guest_port[/(udp|tcp)], e.g. -p 8000:80/tcp)
    #[clap(short, long)]
    port: Vec<PortForward>,
    /// Remove port forwarding rule (e.g. -P 8000:80)
    #[clap(short = 'P', long)]
    rm_port: Vec<PortForward>,
    /// Isolate VM instance from network
    #[clap(long, action = ArgAction::SetTrue)]
    isolate: Option<bool>,
    /// Do not isolate VM instance from network (default)
    #[clap(long, action = ArgAction::SetTrue)]
    no_isolate: Option<bool>,
}

impl Command for ModifyCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();
        let mut instance = instance_store.load(&self.instance)?;

        if instance_store.is_running(&instance) {
            console.info("Note: changes will be applied after the next restart.");
        }

        if let Some(cpus) = &self.cpus {
            instance.cpus = *cpus;
        }

        if let Some(memory) = &self.memory {
            instance.mem = memory.clone();
        }

        if let Some(disk) = &self.disk {
            instance_store.resize(&mut instance, disk.get_bytes() as u64)?;
        }

        if let Some(isolate) = self.isolate {
            instance.isolate = isolate;
        }

        if let Some(no_isolate) = self.no_isolate {
            instance.isolate = !no_isolate;
        }

        instance.hostfwd.append(&mut self.port.clone());
        instance.hostfwd.retain(|p| !self.rm_port.contains(p));

        instance_store.store(&instance)?;
        Result::Ok(())
    }
}
