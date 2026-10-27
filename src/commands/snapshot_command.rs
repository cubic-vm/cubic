use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::SnapshotName;
use clap::Parser;

/// Create a snapshot of a VM instance
///
/// A snapshot saves the disk of a VM instance so it can be restored later with
/// `cubic restore`. The VM instance has to be stopped first. Snapshots of the
/// same instance share every block that did not change, so keeping several of
/// them stays cheap.
///
/// The settings of a VM instance, such as vCPUs, memory and forwarded ports, are
/// not part of a snapshot and are never rolled back.
///
/// Examples:
///
///   Snapshot the VM instance 'my-instance' as 'clean':
///   $ cubic snapshot my-instance/clean
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct SnapshotCommand {
    /// Snapshot of a virtual machine instance, written as <INSTANCE>/<SNAPSHOT>
    #[clap(value_name = "INSTANCE/SNAPSHOT")]
    pub snapshot: SnapshotName,
}

impl Command for SnapshotCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let instance_store = context.get_instance_store();
        let instance_name = self.snapshot.get_instance();

        let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;

        if instance_store.is_running(&instance) {
            return Err(Error::InstanceNotStopped(instance_name.to_string()));
        }

        instance_store.create_snapshot(&instance, self.snapshot.as_str())?;

        context
            .get_console()
            .print(&format!("Created snapshot {}", self.snapshot));
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::Instance;
    use crate::platform::SystemMock;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    fn build_command(target: &str) -> SnapshotCommand {
        SnapshotCommand {
            snapshot: SnapshotName::from_str(target).unwrap(),
        }
    }

    fn build_context(running: &[&str]) -> (commands::Context, Arc<Mutex<Vec<String>>>) {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        builder.get_store().set_running(running);
        let snapshots = Arc::clone(&builder.get_store().snapshots);
        (builder.build(), snapshots)
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let (context, snapshots) = build_context(&[]);

        build_command("test/clean").run(&context).await.unwrap();

        assert_eq!(*snapshots.lock().unwrap(), vec!["create test/clean"]);
    }

    #[tokio::test]
    async fn test_reject_a_running_instance() {
        let (context, _) = build_context(&["test"]);

        assert!(matches!(
            build_command("test/clean").run(&context).await,
            Err(Error::InstanceNotStopped(name)) if name == "test"
        ));
    }
}
