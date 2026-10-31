use crate::actions::{LoadInstanceAction, StopInstanceAction};
use crate::commands::{self, Command};
use crate::error::Result;
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
    async fn wait_until_stopped(context: &commands::Context, stopping: &[Instance]) {
        while stopping.iter().any(|i| context.get_monitor(i).is_running()) {
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    async fn wait_or_kill(
        context: &commands::Context,
        stopping: &[Instance],
        timeout: Duration,
    ) -> Result<()> {
        let wait = Self::wait_until_stopped(context, stopping);

        if tokio::time::timeout(timeout, wait).await.is_err() {
            // The guest ignored the shutdown request, so ask QEMU to quit
            for instance in stopping {
                let mut monitor = context.get_monitor(instance);
                if monitor.is_running() {
                    monitor.kill()?;
                    context.get_console().warn(&format!(
                        "{} did not shut down in {}s and was forced to quit",
                        instance.name,
                        timeout.as_secs()
                    ));
                }
            }
            Self::wait_until_stopped(context, stopping).await;
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
            if context.get_monitor(&instance).is_running() {
                stopping.push(instance);
            }
        }

        let names = stopping
            .iter()
            .map(|instance| instance.name.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let message = format!("Stopping {names}");
        let _spinner = Spinner::new(Arc::clone(context.get_console()), message);

        // Stop instances
        for instance in &stopping {
            StopInstanceAction::new(instance).run(context, self.kill)?;
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
    use crate::commands::ContextBuilder;
    use crate::error::Error;
    use crate::platform::SystemMock;

    #[tokio::test]
    async fn test_require_instance_names_unless_all_is_given() {
        let context = ContextBuilder::new(SystemMock::new()).build();
        let build_command = |all: bool| StopCommand {
            all: all.into(),
            wait: false,
            kill: false,
            instances: Vec::new().into(),
        };

        assert!(matches!(
            build_command(false).run(&context).await,
            Err(Error::MissingInstanceName)
        ));
        assert!(build_command(true).run(&context).await.is_ok());
    }
}
