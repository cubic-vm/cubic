use crate::actions::StartInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::instance::InstanceStore;
use crate::models::{DataSize, HOST_MEMORY_RESERVE, Instance, ResourceAllocator};
use crate::ssh_cmd::PortChecker;
use crate::util;
use crate::view::Console;
use crate::view::Spinner;
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Start VM instances
///
/// Examples:
///
///   Start the VM instance 'my-instance'
///   $ cubic start my-instance
///
///   Start and wait for the VM instance 'my-instance' to start
///   $ cubic start --wait my-instance
///
///   Start multiple VM instances
///   $ cubic start trixie noble
///
///   Pass additional arguments to QEMU
///   $ cubic start trixie --qemu-args="-sandbox on"
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct StartCommand {
    /// Pass additional QEMU arguments
    #[clap(long)]
    pub qemu_args: Option<String>,
    /// Wait until the VM instance has started
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    #[clap(flatten)]
    pub yes: commands::YesArg,
    /// Name of the virtual machine instances to start
    pub instances: Vec<String>,
}

impl Command for StartCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let instance_store = context.get_instance_store();

        let verbosity = console.get_verbosity();
        let port_checker = PortChecker::new();

        // Launch virtual machine instances
        let mut actions = Vec::new();
        for name in &self.instances {
            let instance = &mut instance_store.load(name)?;
            if !instance_store.is_running(instance) {
                if port_checker.is_open(instance.ssh_port) {
                    instance.ssh_port = port_checker.get_new_port()?;
                    instance_store.store(instance)?;
                }

                self.fit_to_available_memory(console, instance_store, instance)?;

                let mut action = StartInstanceAction::new(instance);
                action.run(context, &self.qemu_args, verbosity.is_verbose())?;

                actions.push(action);
            }
        }

        // Wait for virtual machine instances to be started
        if self.wait {
            console.play(Arc::new(Mutex::new(Spinner::new(
                "Starting instance(s)".to_string(),
            ))));
            let deadline = Instant::now() + Duration::from_secs(300);
            while actions.iter().any(|a| !a.is_done()) {
                if Instant::now() >= deadline {
                    console.stop();
                    return Err(Error::StartTimeout);
                }
                sleep(Duration::from_secs(1));
            }
            console.stop()
        }

        Ok(())
    }
}

impl StartCommand {
    /// Reduce an instance to a size that fits the host's available memory.
    ///
    /// QEMU fails to start when the host cannot back the requested memory, so
    /// this proposes the largest resource level that fits the available memory
    /// minus a host reserve. The reduced size is persisted on accept. The start
    /// is aborted when the user declines or nothing fits.
    fn fit_to_available_memory(
        &self,
        console: &mut dyn Console,
        instance_store: &dyn InstanceStore,
        instance: &mut Instance,
    ) -> Result<()> {
        let mut system = System::new();
        system.refresh_memory();
        let available = system.available_memory() as usize;

        if available.saturating_sub(HOST_MEMORY_RESERVE) >= instance.mem.get_bytes() {
            return Ok(());
        }

        let (cpus, mem) = ResourceAllocator::get_resources_for_budget(available)
            .ok_or_else(|| Error::NotEnoughMemory(instance.name.clone()))?;
        let cpus = cpus.min(instance.cpus);

        console.info(&format!(
            "Instance '{}' requests {} vCPUs and {} but only {} is available.\nIt can be started with {} vCPUs and {} instead.",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            DataSize::new(available).to_size(),
            cpus,
            mem.to_size(),
        ));

        if self.yes.value || util::confirm("Reduce and start? [y/n]: ") {
            instance.cpus = cpus;
            instance.mem = mem;
            instance_store.store(instance)?;
            Ok(())
        } else {
            Err(Error::NotEnoughMemory(instance.name.clone()))
        }
    }
}
