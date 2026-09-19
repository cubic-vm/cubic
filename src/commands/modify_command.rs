use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::{DataSize, MIN_DISK, PortForward, ResourceAllocator};
use clap::{ArgAction, Parser};

/// Modify a VM instance
///
/// Use this command to change the settings of an existing VM instance (vCPU, memory,
/// disk, etc.). Port forwarding rules (--port/--rm-port) take effect immediately if
/// the instance is running. All other changes are applied on the next (re-)start of
/// the VM instance.
///
/// Examples:
///
///   Assign 8 vCPUs to a VM instance:
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
    #[clap(flatten)]
    instance: commands::InstanceArg,
    /// Number of vCPUs for the virtual machine instance
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
    #[clap(long, overrides_with = "no_isolate", action = ArgAction::SetTrue)]
    isolate: bool,
    /// Do not isolate VM instance from network (default)
    #[clap(long, overrides_with = "isolate", action = ArgAction::SetTrue)]
    no_isolate: bool,
}

impl Command for ModifyCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
        let instance_store = context.get_instance_store();
        let mut instance = LoadInstanceAction::new().run(context, self.instance.value.as_str())?;

        let is_running = instance_store.is_running(&instance);
        let hostfwd_changed = !self.port.is_empty() || !self.rm_port.is_empty();

        if is_running && hostfwd_changed {
            let mut monitor = instance_store.get_monitor(&instance)?;
            for fwd in &self.rm_port {
                monitor.remove_hostfwd(fwd)?;
            }
            for fwd in &self.port {
                monitor.add_hostfwd(fwd)?;
            }
        }

        if is_running {
            console.info("Note: changes may require a restart to take effect.");
        }

        if let Some(cpus) = &self.cpus {
            instance.cpus = *cpus;
        }

        if let Some(memory) = &self.memory {
            instance.mem = memory.clone();
        }

        for warning in ResourceAllocator::enforce_minimums(
            &mut instance.cpus,
            &mut instance.mem,
            &mut instance.disk_capacity,
        ) {
            console.warn(&warning);
        }

        if let Some(disk) = &self.disk {
            let mut size = disk.get_bytes() as u64;
            if size < MIN_DISK as u64 {
                console.warn(&format!(
                    "Disk raised from {} to {} (minimum usable value).",
                    disk.to_size(),
                    DataSize::new(MIN_DISK).to_size()
                ));
                size = MIN_DISK as u64;
            }
            instance_store.resize(&mut instance, size)?;
        }

        if self.isolate {
            instance.isolate = true;
        } else if self.no_isolate {
            instance.isolate = false;
        }

        instance.hostfwd.append(&mut self.port.clone());
        instance.hostfwd.retain(|p| !self.rm_port.contains(p));

        instance_store.store(&instance)?;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, UserName};
    use crate::platform::{System, SystemMock};
    use crate::view::Console;
    use std::str::FromStr;
    use std::sync::Arc;

    const GIB: usize = 1024 * 1024 * 1024;

    fn build_context(
        system: &Arc<SystemMock>,
        instance_store: InstanceStoreMock,
    ) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Arc::new(SystemMock::new()),
            Console::new(Arc::clone(system) as Arc<dyn System>),
            env,
            Box::new(instance_store),
        )
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(ModifyCommand::try_parse_from(["modify", "../../etc"]).is_err());
    }

    #[tokio::test]
    async fn test_modify_stopped_instance_prints_nothing() {
        let system = Arc::new(SystemMock::new());
        let context = build_context(
            &system,
            InstanceStoreMock::new(vec![Instance {
                name: "test".to_string(),
                cpus: 2,
                mem: DataSize::new(GIB),
                disk_capacity: DataSize::new(GIB),
                ..Instance::default()
            }]),
        );

        ModifyCommand::try_parse_from(["modify", "test", "--cpus", "2"])
            .unwrap()
            .run(&context)
            .await
            .unwrap();

        assert_eq!(system.get_output(), "");
    }

    #[tokio::test]
    async fn test_modify_running_instance_notes_restart() {
        let system = Arc::new(SystemMock::new());
        let context = build_context(
            &system,
            InstanceStoreMock::new_with_running(
                vec![Instance {
                    name: "test".to_string(),
                    cpus: 2,
                    mem: DataSize::new(GIB),
                    disk_capacity: DataSize::new(GIB),
                    ..Instance::default()
                }],
                &["test"],
            ),
        );

        ModifyCommand::try_parse_from(["modify", "test", "--cpus", "2"])
            .unwrap()
            .run(&context)
            .await
            .unwrap();

        assert_eq!(
            system.get_output(),
            "info: Note: changes may require a restart to take effect.\n"
        );
    }

    #[tokio::test]
    async fn test_modify_running_instance_port_attempts_live_apply() {
        let system = Arc::new(SystemMock::new());
        let context = build_context(
            &system,
            InstanceStoreMock::new_with_running(
                vec![Instance {
                    name: "test".to_string(),
                    ..Instance::default()
                }],
                &["test"],
            ),
        );

        // InstanceStoreMock has no real monitor, so a live hostfwd change on
        // a running instance surfaces the mock's InstanceNotRunning error
        // instead of silently deferring to a restart, and the restart note
        // is never printed.
        let result = ModifyCommand::try_parse_from(["modify", "test", "--port", "8080:80"])
            .unwrap()
            .run(&context)
            .await;

        assert!(result.is_err());
        assert_eq!(system.get_output(), "");
    }
}
