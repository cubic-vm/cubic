use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::SnapshotName;
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
        let instance_store = context.get_instance_store();
        let instance_name = self.snapshot.get_instance();
        let snapshot_name = self.snapshot.as_str();

        let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;

        if !instance.has_snapshot(snapshot_name) {
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

        instance_store.restore_snapshot(&instance, snapshot_name)?;

        console.print(&format!("Successfully restored {}", self.snapshot));
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, Instance, Snapshot, UserName};
    use crate::platform::{System, SystemMock};
    use crate::view::Console;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};

    fn build_command(target: &str, yes: bool) -> RestoreCommand {
        RestoreCommand {
            yes: commands::YesArg { value: yes },
            snapshot: SnapshotName::from_str(target).unwrap(),
        }
    }

    fn build_context(instances: Vec<Instance>) -> (commands::Context, Arc<Mutex<Vec<String>>>) {
        build_context_with_system(&Arc::new(SystemMock::new()), instances)
    }

    // The console reads its replies from the given system, so a test that
    // queues an answer has to hand over the same mock it queued it on.
    fn build_context_with_system(
        system: &Arc<SystemMock>,
        instances: Vec<Instance>,
    ) -> (commands::Context, Arc<Mutex<Vec<String>>>) {
        let store = InstanceStoreMock::new(instances);
        let snapshots = Arc::clone(&store.snapshots);
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        (
            commands::Context::new(
                Arc::new(SystemMock::new()),
                Console::new(Arc::clone(system) as Arc<dyn System>),
                env,
                Box::new(store),
            ),
            snapshots,
        )
    }

    fn build_instance(snapshots: Vec<&str>) -> Instance {
        Instance {
            name: "test".to_string(),
            snapshots: snapshots
                .iter()
                .map(|name| Snapshot {
                    name: name.to_string(),
                })
                .collect(),
            ..Instance::default()
        }
    }

    #[tokio::test]
    async fn test_restore_snapshot() {
        let (context, snapshots) = build_context(vec![build_instance(vec!["clean"])]);

        build_command("test/clean", true)
            .run(&context)
            .await
            .unwrap();

        assert_eq!(*snapshots.lock().unwrap(), vec!["restore test/clean"]);
    }

    #[tokio::test]
    async fn test_a_declined_confirmation_leaves_the_disk_alone() {
        let system = Arc::new(SystemMock::new());
        system.push_input("n");
        let (context, snapshots) =
            build_context_with_system(&system, vec![build_instance(vec!["clean"])]);

        build_command("test/clean", false)
            .run(&context)
            .await
            .unwrap();

        assert!(snapshots.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_reject_an_unknown_instance() {
        let (context, _) = build_context(Vec::new());

        assert!(matches!(
            build_command("test/clean", true).run(&context).await,
            Err(Error::UnknownInstance(name)) if name == "test"
        ));
    }

    #[tokio::test]
    async fn test_reject_an_unknown_snapshot() {
        let (context, _) = build_context(vec![build_instance(vec!["deps"])]);

        assert!(matches!(
            build_command("test/clean", true).run(&context).await,
            Err(Error::UnknownSnapshot(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
    }
}
