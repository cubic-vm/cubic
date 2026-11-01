use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::SnapshotName;
use crate::qemu::QemuImg;
use crate::view::ConfirmDialog;
use clap::Parser;

/// Restore a VM instance from a snapshot
///
/// This rolls the disk of the VM instance back to the state it had when the
/// snapshot was taken, including its size. Everything written since then is
/// lost. A running VM instance is stopped first.
///
/// The settings of a VM instance, such as vCPUs, memory and forwarded ports, are
/// not part of a snapshot and stay as they are.
///
/// Examples:
///
///   Restore the VM instance 'my-instance' from the snapshot 'clean':
///   $ cubic restore my-instance/clean
///
///   Restore without confirmation:
///   $ cubic restore --yes my-instance/clean
///
///   List the snapshots of a VM instance:
///   $ cubic show my-instance
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct RestoreCommand {
    #[clap(flatten)]
    pub yes: commands::YesArg,
    /// Snapshot of a virtual machine instance, written as <INSTANCE>/<SNAPSHOT>
    #[clap(value_name = "INSTANCE/SNAPSHOT")]
    pub snapshot: SnapshotName,
}

impl Command for RestoreCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
        let instance_name = self.snapshot.get_instance();
        let snapshot_name = self.snapshot.as_str();

        let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;

        let image = context.get_env().get_instance_image_file(&instance.name);
        let qemu_img = QemuImg::new(context.get_system());
        if !qemu_img.has_snapshot(&image, snapshot_name) {
            return Err(Error::UnknownSnapshot(
                instance_name.to_string(),
                snapshot_name.to_string(),
            ));
        }

        console.info("The instance is stopped and all changes since the snapshot are lost.");

        if !self.yes.value && !ConfirmDialog::new("Do you want to proceed?").confirm(console) {
            return Ok(0);
        }

        // The restore discards the current state, so kill instead of a clean stop.
        commands::StopCommand {
            all: false.into(),
            wait: true,
            kill: true,
            instances: vec![instance_name.clone()].into(),
        }
        .run(context)
        .await?;

        qemu_img.restore_snapshot(&image, snapshot_name)?;

        console.print(&format!("Successfully restored {}", self.snapshot));
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

    fn build_command(target: &str, yes: bool) -> RestoreCommand {
        RestoreCommand {
            yes: commands::YesArg { value: yes },
            snapshot: SnapshotName::from_str(target).unwrap(),
        }
    }

    // Returns the context and a check whether test/clean was restored
    fn build_context(
        system: SystemMock,
        snapshots: &[&str],
    ) -> (commands::Context, impl Fn() -> bool) {
        let builder = ContextBuilder::new(system);
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        builder.add_image_info("test", 0, 0, snapshots);
        let restore = builder.add_snapshot_command("-a", "test", "clean");
        let system = Arc::clone(builder.get_system());
        let was_restored = move || system.get_executed_commands().contains(&restore);
        (builder.build(), was_restored)
    }

    #[tokio::test]
    async fn test_restore_snapshot() {
        let (context, was_restored) = build_context(SystemMock::new(), &["clean"]);

        build_command("test/clean", true)
            .run(&context)
            .await
            .unwrap();

        assert!(was_restored());
    }

    #[tokio::test]
    async fn test_a_declined_confirmation_leaves_the_disk_alone() {
        let system = SystemMock::new();
        system.push_input("n");
        let (context, was_restored) = build_context(system, &["clean"]);

        build_command("test/clean", false)
            .run(&context)
            .await
            .unwrap();

        assert!(!was_restored());
    }

    #[tokio::test]
    async fn test_reject_an_unknown_snapshot() {
        let (context, _) = build_context(SystemMock::new(), &["deps"]);

        assert!(matches!(
            build_command("test/clean", true).run(&context).await,
            Err(Error::UnknownSnapshot(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
    }
}
