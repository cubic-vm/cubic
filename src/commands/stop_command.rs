use crate::actions::{LoadInstanceAction, StopInstanceAction};
use crate::commands::{self, Command};
use crate::error::Result;
use crate::instance::InstanceStore;
use crate::models::Instance;
use crate::view::Spinner;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;

/// Stop VM instances
///
/// Examples:
///
///   Stop the VM instance 'my-instance':
///   $ cubic stop my-instance
///
///   Stop and wait until the VM instance 'my-instance' has stopped:
///   $ cubic stop --wait my-instance
///
///   Stop all VM instances:
///   $ cubic stop --all --wait
///
///   Force-kill the VM instance 'my-instance':
///   $ cubic stop --kill my-instance
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct StopCommand {
    #[clap(flatten)]
    pub all: commands::AllInstancesArg,
    /// Wait for the virtual machine instance to be stopped
    #[clap(short, long, default_value_t = false)]
    pub wait: bool,
    /// Kill the virtual machine instance
    #[clap(short, long, default_value_t = false)]
    pub kill: bool,
    #[clap(flatten)]
    pub instances: commands::InstancesArg,
}

const STOP_TIMEOUT: Duration = Duration::from_secs(60);

impl StopCommand {
    async fn wait_until_stopped(instance_store: &dyn InstanceStore, stopping: &[Instance]) {
        while stopping.iter().any(|i| instance_store.is_running(i)) {
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    async fn wait_or_kill(
        context: &commands::Context,
        stopping: &[Instance],
        timeout: Duration,
    ) -> Result<()> {
        let instance_store = context.get_instance_store();
        let wait = Self::wait_until_stopped(instance_store, stopping);

        if tokio::time::timeout(timeout, wait).await.is_err() {
            // The guest ignored the shutdown request, so ask QEMU to quit
            for instance in stopping {
                if instance_store.is_running(instance) {
                    instance_store.kill(instance)?;
                    context.get_console().warn(&format!(
                        "{} did not shut down in {}s and was forced to quit",
                        instance.name,
                        timeout.as_secs()
                    ));
                }
            }
            Self::wait_until_stopped(instance_store, stopping).await;
        }

        Ok(())
    }
}

impl Command for StopCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let instance_store = context.get_instance_store();

        if !self.all.value {
            self.instances.require_names()?;
        }

        let stop_instances = if self.all.value {
            instance_store.get_instances()
        } else {
            self.instances.get_names()
        };

        // Only stop instances that are running
        let mut stopping = Vec::new();
        for name in &stop_instances {
            let instance = LoadInstanceAction::new().run(context, name)?;
            if instance_store.is_running(&instance) {
                stopping.push(instance);
            }
        }

        let names = stopping
            .iter()
            .map(|instance| instance.name.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let _spinner = Spinner::new(
            Arc::clone(context.get_console()),
            format!("Stopping {names}"),
        );

        // Stop instances
        for instance in &stopping {
            StopInstanceAction::new(instance).run(instance_store, self.kill)?;
        }

        if self.wait {
            Self::wait_or_kill(context, &stopping, STOP_TIMEOUT).await?;
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, UserName};
    use crate::platform::SystemMock;
    use crate::view::Console;
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_context(store: InstanceStoreMock) -> commands::Context {
        let env = Environment::new(
            UserName::from_str("myuser").unwrap(),
            String::new(),
            String::new(),
        );
        commands::Context::new(
            Arc::new(SystemMock::new()),
            Console::new(Arc::new(SystemMock::new())),
            env,
            Box::new(store),
        )
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(StopCommand::try_parse_from(["stop", "../../etc"]).is_err());
    }

    #[tokio::test]
    async fn test_reject_empty_instance_list_without_all() {
        let context = build_context(InstanceStoreMock::new(Vec::new()));

        assert!(matches!(
            StopCommand {
                all: false.into(),
                wait: false,
                kill: false,
                instances: Vec::new().into(),
            }
            .run(&context)
            .await,
            Err(Error::MissingInstanceName)
        ));
    }

    #[tokio::test]
    async fn test_allow_empty_instance_list_with_all() {
        let context = build_context(InstanceStoreMock::new(Vec::new()));

        assert!(
            StopCommand {
                all: true.into(),
                wait: false,
                kill: false,
                instances: Vec::new().into(),
            }
            .run(&context)
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_wait_forces_a_stuck_instance_to_quit() {
        let instance = Instance {
            name: "web".to_string(),
            ..Instance::default()
        };
        let store = InstanceStoreMock::new_with_running(vec![instance.clone()], &["web"]);
        let killed = Arc::clone(&store.killed);
        let context = build_context(store);

        StopCommand::wait_or_kill(&context, &[instance], Duration::ZERO)
            .await
            .unwrap();

        assert_eq!(*killed.lock().unwrap(), ["web"]);
    }
}
