use crate::actions::{LoadInstanceAction, StartInstanceAction, StoreInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::{DataSize, HOST_MEMORY_RESERVE, Instance, ResourceAllocator};
use crate::ssh::PortChecker;
use crate::view::ConfirmDialog;
use crate::view::Spinner;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;

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
    #[clap(flatten)]
    pub accel: commands::AccelArg,
    /// Wait until the VM instance has started
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    #[clap(flatten)]
    pub yes: commands::YesArg,
    #[clap(flatten)]
    pub instances: commands::InstancesArg,
}

impl Command for StartCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
        self.instances.require_names()?;

        let port_checker = PortChecker::new();

        // Launch virtual machine instances
        let mut actions = Vec::new();
        let mut starting = Vec::new();
        for name in &self.instances.value {
            let instance = &mut LoadInstanceAction::new().run(context, name.as_str())?;
            if !context.get_monitor(instance).is_running() {
                if port_checker
                    .is_open(context.get_system(), instance.ssh_port)
                    .await
                {
                    let old_port = instance.ssh_port;
                    instance.ssh_port = context.get_system().bind_port()?;
                    StoreInstanceAction::new().run(context, instance)?;
                    console.debug(&format!(
                        "Instance '{}' ssh_port {} is taken, reassigned to {}",
                        instance.name, old_port, instance.ssh_port
                    ));
                }

                self.fit_to_host(context, instance)?;

                let mut action = StartInstanceAction::new(instance);
                action.run(context, &self.qemu_args, self.accel.value)?;

                actions.push(action);
                // Only the instances that are launched are named
                starting.push(instance.name.clone());
            }
        }

        // Wait for virtual machine instances to be started
        if self.wait && !starting.is_empty() {
            let text = format!("Starting {}", starting.join(", "));
            let _spinner = Spinner::new(Arc::clone(console), text);
            let wait = async {
                for action in &actions {
                    while !action.is_done(context).await {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            };
            if tokio::time::timeout(Duration::from_secs(300), wait)
                .await
                .is_err()
            {
                return Err(Error::StartTimeout);
            }
        }

        Ok(0)
    }
}

impl StartCommand {
    /// Reduce an instance to a size that fits the host's available memory and
    /// CPU count.
    ///
    /// QEMU fails to start when the host cannot back the requested memory or
    /// CPU count, so this proposes the largest resource level that fits the
    /// available memory minus a host reserve, capped at the host's CPU count.
    /// The reduced size is persisted on accept. The start is aborted when the
    /// user declines or nothing fits.
    fn fit_to_host(&self, context: &commands::Context, instance: &mut Instance) -> Result<()> {
        let console = context.get_console();
        let system = context.get_system();
        let available = system.get_available_memory() as usize;
        let host_cpus = system.get_cpu_count();

        console.debug(&format!(
            "Instance '{}' requests {} vCPUs and {}, host has {} vCPUs and {} available with {} reserved",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            host_cpus,
            DataSize::new(available).to_size(),
            DataSize::new(HOST_MEMORY_RESERVE).to_size(),
        ));

        let mem_fits = available.saturating_sub(HOST_MEMORY_RESERVE) >= instance.mem.get_bytes();
        let cpus_fit = instance.cpus <= host_cpus;

        if mem_fits && cpus_fit {
            return Ok(());
        }

        let (cpus, mem) = ResourceAllocator::get_resources_for_budget(available)
            .ok_or_else(|| Error::NotEnoughHostResources(instance.name.clone()))?;
        let cpus = cpus.min(instance.cpus).min(host_cpus);

        console.warn(&format!(
            "Instance '{}' requests {} vCPUs and {} but the host has {} vCPUs and {} available.\nIt can be started with {} vCPUs and {} instead.",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            host_cpus,
            DataSize::new(available).to_size(),
            cpus,
            mem.to_size(),
        ));

        if self.yes.value || ConfirmDialog::new("Reduce and start?").confirm(console) {
            instance.cpus = cpus;
            instance.mem = mem;
            StoreInstanceAction::new().run(context, instance)?;
            Ok(())
        } else {
            Err(Error::NotEnoughHostResources(instance.name.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{Context, ContextBuilder};
    use crate::platform::SystemMock;

    const GIB: usize = 1024 * 1024 * 1024;

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            cpus: 8,
            mem: DataSize::new(8 * GIB),
            ..Instance::default()
        }
    }

    // Seeds a stopped instance on the given ssh port and hands back a context
    // over a host too small to run it. The run then stops at the memory check,
    // which is the step right after the port reassignment, so a port assertion
    // never depends on what QEMU would have done later.
    fn build_starved_context(system: SystemMock, ssh_port: u16) -> Context {
        let system = system.set_host_resources(GIB as u64, GIB as u64, 8);
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![Instance {
            ssh_port,
            ..build_instance()
        }]);
        builder.build()
    }

    // Fits the instance from `build_instance` to the host of `system`
    fn fit_to_host(system: SystemMock, args: &[&str]) -> (Result<()>, Instance) {
        let context = ContextBuilder::new(system).build();
        let command = StartCommand::try_parse_from(args).unwrap();
        let mut instance = build_instance();
        let result = command.fit_to_host(&context, &mut instance);
        (result, instance)
    }

    #[tokio::test]
    async fn test_reassigns_an_ssh_port_that_is_taken() {
        let context = build_starved_context(SystemMock::new().add_open_port(22000), 22000);
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();

        assert!(matches!(
            command.run(&context).await,
            Err(Error::NotEnoughHostResources(_))
        ));
        // Read back from the config, so the new port has to have been written
        // rather than only set on the instance in hand.
        let instance = LoadInstanceAction::new().read(&context, "test").unwrap();
        assert_ne!(instance.ssh_port, 22000);
    }

    #[tokio::test]
    async fn test_keeps_an_ssh_port_that_is_free() {
        let context = build_starved_context(SystemMock::new(), 22000);
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();

        assert!(matches!(
            command.run(&context).await,
            Err(Error::NotEnoughHostResources(_))
        ));
        let instance = LoadInstanceAction::new().read(&context, "test").unwrap();
        assert_eq!(instance.ssh_port, 22000);
    }

    #[test]
    fn test_keeps_size_when_memory_is_available() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (16 * GIB) as u64, 8);

        let (result, instance) = fit_to_host(system, &["start", "--yes", "test"]);

        result.unwrap();
        assert_eq!(instance.cpus, 8);
        assert_eq!(instance.mem.get_bytes(), 8 * GIB);
    }

    #[test]
    fn test_reduces_cpus_when_cpus_alone_exceed_the_host() {
        let system = SystemMock::new().set_host_resources((32 * GIB) as u64, (32 * GIB) as u64, 4);

        let (result, instance) = fit_to_host(system, &["start", "--yes", "test"]);

        result.unwrap();
        assert_eq!(instance.cpus, 4);
    }

    #[test]
    fn test_reduces_size_to_fit_available_memory() {
        // 5 GiB available minus the 1 GiB reserve leaves a 4 GiB budget.
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);

        let (result, instance) = fit_to_host(system, &["start", "--yes", "test"]);

        result.unwrap();
        assert_eq!(instance.cpus, 8);
        assert_eq!(instance.mem.get_bytes(), 4 * GIB);
    }

    #[test]
    fn test_reduces_size_when_the_user_confirms() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);
        system.push_input("y");

        let (result, instance) = fit_to_host(system, &["start", "test"]);

        result.unwrap();
        assert_eq!(instance.mem.get_bytes(), 4 * GIB);
    }

    #[test]
    fn test_errors_when_the_user_declines() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);
        system.push_input("n");

        let (result, instance) = fit_to_host(system, &["start", "test"]);

        assert!(matches!(
            result,
            Err(Error::NotEnoughHostResources(name)) if name == "test"
        ));
        // The instance keeps its size, the reduction is only applied on accept.
        assert_eq!(instance.mem.get_bytes(), 8 * GIB);
    }

    #[test]
    fn test_errors_when_nothing_fits() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, GIB as u64, 8);

        let (result, _) = fit_to_host(system, &["start", "--yes", "test"]);

        assert!(matches!(
            result,
            Err(Error::NotEnoughHostResources(name)) if name == "test"
        ));
    }
}
