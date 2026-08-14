use crate::actions::{LoadInstanceAction, StartInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::instance::InstanceStore;
use crate::models::{DataSize, HOST_MEMORY_RESERVE, Instance, ResourceAllocator};
use crate::platform::System;
use crate::ssh::PortChecker;
use crate::view::Console;
use crate::view::{ConfirmDialog, Spinner};
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

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
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        self.instances.require_names()?;

        let instance_store = context.get_instance_store();

        let port_checker = PortChecker::new();

        // Launch virtual machine instances
        let mut actions = Vec::new();
        let mut starting = Vec::new();
        for name in &self.instances.value {
            let instance = &mut LoadInstanceAction::new().run(context, console, name.as_str())?;
            if !instance_store.is_running(instance) {
                if port_checker.is_open(context.get_system(), instance.ssh_port) {
                    let old_port = instance.ssh_port;
                    instance.ssh_port = context.get_system().bind_port()?;
                    instance_store.store(instance)?;
                    console.debug(&format!(
                        "Instance '{}' ssh_port {} is taken, reassigned to {}",
                        instance.name, old_port, instance.ssh_port
                    ));
                }

                self.fit_to_available_memory(
                    console,
                    context.get_system(),
                    instance_store,
                    instance,
                )?;

                let mut action = StartInstanceAction::new(instance);
                action.run(context, &self.qemu_args, self.accel.value, console)?;

                actions.push(action);
                // Only the instances that are launched are named
                starting.push(instance.name.clone());
            }
        }

        // Wait for virtual machine instances to be started
        if self.wait && !starting.is_empty() {
            console.play(Arc::new(Mutex::new(Spinner::new(format!(
                "Starting {}",
                starting.join(", ")
            )))));
            let deadline = Instant::now() + Duration::from_secs(300);
            while actions.iter().any(|a| !a.is_done(context.get_system())) {
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
        console: &mut Console<'_>,
        system: &dyn System,
        instance_store: &dyn InstanceStore,
        instance: &mut Instance,
    ) -> Result<()> {
        let available = system.get_available_memory() as usize;

        console.debug(&format!(
            "Instance '{}' requests {}, host has {} available with {} reserved",
            instance.name,
            instance.mem.to_size(),
            DataSize::new(available).to_size(),
            DataSize::new(HOST_MEMORY_RESERVE).to_size(),
        ));

        if available.saturating_sub(HOST_MEMORY_RESERVE) >= instance.mem.get_bytes() {
            return Ok(());
        }

        let (cpus, mem) = ResourceAllocator::get_resources_for_budget(available)
            .ok_or_else(|| Error::NotEnoughMemory(instance.name.clone()))?;
        let cpus = cpus.min(instance.cpus);

        console.warn(&format!(
            "Instance '{}' requests {} vCPUs and {} but only {} is available.\nIt can be started with {} vCPUs and {} instead.",
            instance.name,
            instance.cpus,
            instance.mem.to_size(),
            DataSize::new(available).to_size(),
            cpus,
            mem.to_size(),
        ));

        if self.yes.value || ConfirmDialog::new("Reduce and start?").confirm(console) {
            instance.cpus = cpus;
            instance.mem = mem;
            instance_store.store(instance)?;
            Ok(())
        } else {
            Err(Error::NotEnoughMemory(instance.name.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Context;
    use crate::instance::{InstanceDao, InstanceStoreMock};
    use crate::models::{Environment, UserName};
    use crate::platform::SystemMock;
    use std::rc::Rc;
    use std::str::FromStr;

    const GIB: usize = 1024 * 1024 * 1024;

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            cpus: 8,
            mem: DataSize::new(8 * GIB),
            ..Instance::default()
        }
    }

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
        )
    }

    // A real dao over a mocked host, so an assertion reads back the port that
    // survived a write rather than one the store was handed.
    fn build_dao(system: &Rc<SystemMock>) -> InstanceDao {
        InstanceDao::new(Rc::clone(system) as Rc<dyn System>, &build_env()).unwrap()
    }

    // Seeds a stopped instance on the given ssh port and hands back a context
    // over a host too small to run it. The run then stops at the memory check,
    // which is the step right after the port reassignment, so a port assertion
    // never depends on what QEMU would have done later.
    fn build_starved_context(system: &Rc<SystemMock>, ssh_port: u16) -> Context {
        let instance = Instance {
            ssh_port,
            ..build_instance()
        };
        build_dao(system).store(&instance).unwrap();

        Context::new(
            Rc::clone(system) as Rc<dyn System>,
            build_env(),
            Box::new(build_dao(system)),
        )
    }

    #[test]
    fn test_reassigns_an_ssh_port_that_is_taken() {
        let system = Rc::new(
            SystemMock::new()
                .set_host_resources(GIB as u64, GIB as u64, 8)
                .add_dir("/data/machines/test")
                .add_open_port(22000),
        );
        let context = build_starved_context(&system, 22000);
        let mut console = Console::new(system.as_ref());
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();

        assert!(matches!(
            command.run(&mut console, &context),
            Err(Error::NotEnoughMemory(_))
        ));
        // Read back through the dao, so the new port has to have been written
        // rather than only set on the instance in hand.
        assert_ne!(build_dao(&system).load("test").unwrap().ssh_port, 22000);
    }

    #[test]
    fn test_keeps_an_ssh_port_that_is_free() {
        let system = Rc::new(
            SystemMock::new()
                .set_host_resources(GIB as u64, GIB as u64, 8)
                .add_dir("/data/machines/test"),
        );
        let context = build_starved_context(&system, 22000);
        let mut console = Console::new(system.as_ref());
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();

        assert!(matches!(
            command.run(&mut console, &context),
            Err(Error::NotEnoughMemory(_))
        ));
        assert_eq!(build_dao(&system).load("test").unwrap().ssh_port, 22000);
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(StartCommand::try_parse_from(["start", "../../etc"]).is_err());
    }

    #[test]
    fn test_keeps_size_when_memory_is_available() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (16 * GIB) as u64, 8);
        let mut console = Console::new(&system);
        let store = InstanceStoreMock::new(vec![build_instance()]);
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();
        let mut instance = build_instance();

        command
            .fit_to_available_memory(&mut console, &system, &store, &mut instance)
            .unwrap();

        assert_eq!(instance.cpus, 8);
        assert_eq!(instance.mem.get_bytes(), 8 * GIB);
    }

    #[test]
    fn test_reduces_size_to_fit_available_memory() {
        // 5 GiB available minus the 1 GiB reserve leaves a 4 GiB budget.
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);
        let mut console = Console::new(&system);
        let store = InstanceStoreMock::new(vec![build_instance()]);
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();
        let mut instance = build_instance();

        command
            .fit_to_available_memory(&mut console, &system, &store, &mut instance)
            .unwrap();

        assert_eq!(instance.cpus, 8);
        assert_eq!(instance.mem.get_bytes(), 4 * GIB);
    }

    #[test]
    fn test_reduces_size_when_the_user_confirms() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);
        system.push_input("y");
        let mut console = Console::new(&system);
        let store = InstanceStoreMock::new(vec![build_instance()]);
        let command = StartCommand::try_parse_from(["start", "test"]).unwrap();
        let mut instance = build_instance();

        command
            .fit_to_available_memory(&mut console, &system, &store, &mut instance)
            .unwrap();

        assert_eq!(instance.mem.get_bytes(), 4 * GIB);
    }

    #[test]
    fn test_errors_when_the_user_declines() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (5 * GIB) as u64, 8);
        system.push_input("n");
        let mut console = Console::new(&system);
        let store = InstanceStoreMock::new(vec![build_instance()]);
        let command = StartCommand::try_parse_from(["start", "test"]).unwrap();
        let mut instance = build_instance();

        assert!(matches!(
            command.fit_to_available_memory(&mut console, &system, &store, &mut instance),
            Err(Error::NotEnoughMemory(name)) if name == "test"
        ));
        // The instance keeps its size, the reduction is only applied on accept.
        assert_eq!(instance.mem.get_bytes(), 8 * GIB);
    }

    #[test]
    fn test_errors_when_nothing_fits() {
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, GIB as u64, 8);
        let mut console = Console::new(&system);
        let store = InstanceStoreMock::new(vec![build_instance()]);
        let command = StartCommand::try_parse_from(["start", "--yes", "test"]).unwrap();
        let mut instance = build_instance();

        assert!(matches!(
            command.fit_to_available_memory(&mut console, &system, &store, &mut instance),
            Err(Error::NotEnoughMemory(name)) if name == "test"
        ));
    }
}
