use crate::actions::{LoadInstanceAction, StartInstanceAction, StoreInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::{DataSize, HOST_MEMORY_RESERVE, Instance, MIN_CPUS, ResourceAllocator};
use crate::platform::Child;
use crate::qemu::QemuMonitorClient;
use crate::ssh::PortChecker;
use crate::util::ResourceLock;
use crate::view::ConfirmDialog;
use crate::view::Spinner;
use clap::Parser;
use std::process::Command as ProcessCommand;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};

const LAUNCH_TIMEOUT: Duration = Duration::from_secs(30);

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
            let _lock = ResourceLock::acquire_instances(context, &[name.as_str()])?;
            let instance = &mut LoadInstanceAction::new().run(context, name.as_str())?;
            if !context.is_running(instance) {
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

                let monitor = context.get_monitor(instance);
                let command = self.build_vm_command(context, instance);
                let supervisor = context.get_system().spawn(command, true)?;
                let launched = Self::forward_until_launched(context, &monitor, supervisor);
                if let Ok(false) = tokio::time::timeout(LAUNCH_TIMEOUT, launched).await {
                    return Err(Error::StartFailed(instance.name.clone()));
                }

                actions.push(StartInstanceAction::new(instance));
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
    async fn forward_until_launched(
        context: &commands::Context,
        monitor: &QemuMonitorClient,
        mut supervisor: Box<dyn Child>,
    ) -> bool {
        let console = context.get_console();
        let mut stdout = BufReader::new(supervisor.take_stdout()).lines();
        let mut stderr = BufReader::new(supervisor.take_stderr()).lines();
        let mut tick = tokio::time::interval(Duration::from_millis(50));
        loop {
            tokio::select! {
                // Output first, so a launch is only seen once all of it is shown.
                biased;
                Ok(Some(line)) = stdout.next_line() => console.print(&line),
                Ok(Some(line)) = stderr.next_line() => console.eprint(&line),
                success = supervisor.wait() => {
                    while let Ok(Some(line)) = stdout.next_line().await {
                        console.print(&line);
                    }
                    while let Ok(Some(line)) = stderr.next_line().await {
                        console.eprint(&line);
                    }
                    return success;
                }
                _ = tick.tick() => if monitor.is_running() {
                    return true;
                },
            }
        }
    }

    fn build_vm_command(&self, context: &commands::Context, instance: &Instance) -> ProcessCommand {
        let mut command = ProcessCommand::new(context.get_system().get_current_exe());
        command.arg("vm").arg(&instance.name);
        command.arg("--accel").arg(self.accel.value.get_name());
        if let Some(args) = &self.qemu_args {
            command.arg(format!("--qemu-args={args}"));
        }
        command.args(context.get_console().get_verbosity_arg());
        command
    }

    /// Offers to shrink an instance that exceeds the host, since QEMU fails to
    /// start it otherwise. Skipped when the host size is unknown.
    fn fit_to_host(&self, context: &commands::Context, instance: &mut Instance) -> Result<()> {
        let console = context.get_console();
        let system = context.get_system();
        let available = system.get_available_memory();
        let host_cpus = system.get_cpu_count();
        let (Some(available), Some(host_cpus)) = (available, host_cpus) else {
            console.debug("Host memory or CPU count is unknown, skipping the size check");
            return Ok(());
        };
        let available = available as usize;

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
        let cpus = cpus.min(instance.cpus).min(host_cpus).max(MIN_CPUS);

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

        if self.yes.value || ConfirmDialog::new("Reduce and start?").confirm(console)? {
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
    async fn test_starts_the_instance_through_the_supervisor() {
        let line = "cubic vm test --accel off --qemu-args=-sandbox on -v";
        let system = SystemMock::new().add_command_output(line, b"debug: launched\n");
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![build_instance()]);
        let system = Arc::clone(builder.get_system());
        let context = builder.build();
        context.get_console().set_verbosity(1);
        let args = ["start", "--accel", "off", "--qemu-args=-sandbox on", "test"];
        let command = StartCommand::try_parse_from(args).unwrap();

        command.run(&context).await.unwrap();

        assert_eq!(system.get_executed_commands(), vec![line]);
        assert!(system.get_output().ends_with("debug: launched\n"));
    }

    #[tokio::test]
    async fn test_shows_the_error_of_a_failed_supervisor_and_names_the_instance() {
        let line = "cubic vm test --accel auto";
        let system = SystemMock::new().add_failing_command(line, "error: QEMU not found\n");
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![build_instance()]);
        let system = Arc::clone(builder.get_system());
        let command = StartCommand::try_parse_from(["start", "test"]).unwrap();

        let result = command.run(&builder.build()).await;

        assert!(matches!(result, Err(Error::StartFailed(name)) if name == "test"));
        assert_eq!(system.get_output(), "error: QEMU not found\n");
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
