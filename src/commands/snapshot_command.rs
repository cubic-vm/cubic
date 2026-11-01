use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::SnapshotName;
use crate::qemu::QemuImg;
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
        let instance_name = self.snapshot.get_instance();

        let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;

        if context.get_monitor(&instance).is_running() {
            return Err(Error::InstanceNotStopped(instance_name.to_string()));
        }

        let snapshot = self.snapshot.as_str();
        let image = context.get_env().get_instance_image_file(&instance.name);
        let qemu_img = QemuImg::new(context.get_system());
        if qemu_img.has_snapshot(&image, snapshot) {
            return Err(Error::SnapshotAlreadyExists(
                instance.name.to_string(),
                snapshot.to_string(),
            ));
        }

        qemu_img.create_snapshot(&image, snapshot)?;

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
    use std::sync::Arc;

    fn build_command(target: &str) -> SnapshotCommand {
        SnapshotCommand {
            snapshot: SnapshotName::from_str(target).unwrap(),
        }
    }

    // Returns the context and a check whether test/clean was created
    fn build_context(
        snapshots: &[&str],
        running: &[&str],
    ) -> (commands::Context, impl Fn() -> bool) {
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        builder.add_image_info("test", 0, 0, snapshots);
        builder.set_running(running);
        let create = builder.add_snapshot_command("-c", "test", "clean");
        let system = Arc::clone(builder.get_system());
        let was_created = move || system.get_executed_commands().contains(&create);
        (builder.build(), was_created)
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let (context, was_created) = build_context(&[], &[]);

        build_command("test/clean").run(&context).await.unwrap();

        assert!(was_created());
    }

    #[tokio::test]
    async fn test_reject_an_existing_snapshot() {
        let (context, was_created) = build_context(&["clean"], &[]);

        assert!(matches!(
            build_command("test/clean").run(&context).await,
            Err(Error::SnapshotAlreadyExists(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
        assert!(!was_created());
    }

    #[tokio::test]
    async fn test_reject_a_running_instance() {
        let (context, _) = build_context(&[], &["test"]);

        assert!(matches!(
            build_command("test/clean").run(&context).await,
            Err(Error::InstanceNotStopped(name)) if name == "test"
        ));
    }
}
