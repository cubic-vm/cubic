use crate::actions::{DeleteInstanceAction, LoadInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::models::{InstanceName, SnapshotName};
use crate::qemu::QemuImg;
use crate::util::Either;
use crate::view::ConfirmDialog;
use clap::Parser;
use std::collections::HashSet;

/// Delete VM instances and snapshots
///
/// A target written as <INSTANCE> deletes the whole VM instance including all
/// of its snapshots. A target written as <INSTANCE>/<SNAPSHOT> deletes only
/// that one snapshot and leaves the VM instance alone.
///
/// Examples:
///
///   Delete the VM instance 'my-instance':
///   $ cubic delete my-instance
///
///   Delete multiple VM instances:
///   $ cubic delete trixie noble
///
///   Delete the snapshot 'clean' of the VM instance 'my-instance':
///   $ cubic delete my-instance/clean
///
///   Delete multiple VM instances without confirmation:
///   $ cubic delete --yes trixie noble
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct DeleteCommand {
    /// Delete the VM instances even when running (Deprecated)
    #[clap(hide = true, short, long, default_value_t = false)]
    force: bool,
    #[clap(flatten)]
    yes: commands::YesArg,
    /// Names of the virtual machine instances or their snapshots
    #[clap(value_name = "TARGETS")]
    targets: Vec<Either<InstanceName, SnapshotName>>,
}

impl DeleteCommand {
    fn get_instance(target: &Either<InstanceName, SnapshotName>) -> &InstanceName {
        match target {
            Either::Left(instance) => instance,
            Either::Right(snapshot) => snapshot.get_instance(),
        }
    }

    fn is_instance(target: &Either<InstanceName, SnapshotName>) -> bool {
        matches!(target, Either::Left(_))
    }

    /// The targets to act on. A snapshot whose instance is deleted as a whole is
    /// dropped, because deleting the instance already removes its snapshots.
    /// Duplicates are dropped so a target is never deleted twice.
    fn get_targets(&self) -> Vec<&Either<InstanceName, SnapshotName>> {
        let deleted_instances: HashSet<&InstanceName> = self
            .targets
            .iter()
            .filter(|target| Self::is_instance(target))
            .map(Self::get_instance)
            .collect();

        let mut seen = HashSet::new();
        self.targets
            .iter()
            .filter(|target| {
                Self::is_instance(target) || !deleted_instances.contains(Self::get_instance(target))
            })
            .filter(|target| seen.insert(*target))
            .collect()
    }
}

impl Command for DeleteCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();

        if self.targets.is_empty() {
            return Err(Error::MissingInstanceName);
        }

        for target in &self.targets {
            let instance_name = Self::get_instance(target);
            if !context.has_instance(instance_name.as_str()) {
                return Err(Error::UnknownInstance(instance_name.to_string()));
            }

            if let Either::Right(snapshot_name) = target {
                let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;
                let image = context.get_env().get_instance_image_file(&instance.name);
                if !QemuImg::new(context.get_system()).has_snapshot(&image, snapshot_name.as_str())
                {
                    return Err(Error::UnknownSnapshot(
                        instance_name.to_string(),
                        snapshot_name.as_str().to_string(),
                    ));
                }
            }
        }

        let targets = self.get_targets();

        console.info("Running instances are stopped before deletion.");
        if !self.yes.value && !ConfirmDialog::new("Do you want to proceed?").confirm(console) {
            return Ok(0);
        }

        for target in &targets {
            let instance_name = Self::get_instance(target);

            // Free the disk lock first. A full instance delete can kill it, a
            // snapshot needs the instance shut down cleanly.
            commands::StopCommand {
                all: false.into(),
                wait: true,
                kill: Self::is_instance(target),
                instances: vec![instance_name.clone()].into(),
            }
            .run(context)
            .await?;

            let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;
            match target {
                Either::Right(snapshot_name) => {
                    let image = context.get_env().get_instance_image_file(&instance.name);
                    QemuImg::new(context.get_system())
                        .delete_snapshot(&image, snapshot_name.as_str())?
                }
                Either::Left(_) => DeleteInstanceAction::new().run(context, &instance)?,
            }
            console.debug(&format!("Deleted {target}"));
        }

        console.print("Successfully deleted all targets");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::{Environment, Instance};
    use crate::platform::{FileSystem, SystemMock};
    use std::sync::Arc;

    struct Recorders {
        system: Arc<SystemMock>,
        // The qemu-img call that deletes test/clean
        delete_clean: String,
    }

    impl Recorders {
        fn has_instance(&self) -> bool {
            let dir = Environment::default().get_instance_dir2("test");
            self.system.exists_path(&dir)
        }

        fn count_snapshot_deletes(&self) -> usize {
            let commands = self.system.get_executed_commands();
            commands.iter().filter(|c| **c == self.delete_clean).count()
        }
    }

    // Builds a context with the instance "test" whose image holds `snapshots`
    fn build_context(snapshots: &[&str]) -> (commands::Context, Recorders) {
        let builder = ContextBuilder::new(SystemMock::new());
        builder.add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        builder.add_image_info("test", 0, 0, snapshots);
        let recorders = Recorders {
            system: Arc::clone(builder.get_system()),
            delete_clean: builder.add_snapshot_command("-d", "test", "clean"),
        };
        (builder.build(), recorders)
    }

    fn build_command(targets: &[&str]) -> DeleteCommand {
        DeleteCommand {
            force: false,
            yes: commands::YesArg { value: true },
            targets: targets
                .iter()
                .map(|target| target.parse().unwrap())
                .collect(),
        }
    }

    #[tokio::test]
    async fn test_delete_instance() {
        let (context, recorders) = build_context(&["clean"]);

        build_command(&["test"]).run(&context).await.unwrap();

        assert!(!recorders.has_instance());
        assert_eq!(recorders.count_snapshot_deletes(), 0);
    }

    #[tokio::test]
    async fn test_delete_snapshot_keeps_the_instance() {
        let (context, recorders) = build_context(&["clean"]);

        build_command(&["test/clean"]).run(&context).await.unwrap();

        assert_eq!(recorders.count_snapshot_deletes(), 1);
        assert!(recorders.has_instance());
    }

    #[tokio::test]
    async fn test_delete_ignores_duplicate_targets() {
        let (context, recorders) = build_context(&["clean"]);

        build_command(&["test/clean", "test/clean"])
            .run(&context)
            .await
            .unwrap();

        assert_eq!(recorders.count_snapshot_deletes(), 1);
    }

    #[tokio::test]
    async fn test_delete_instance_skips_its_own_snapshot() {
        let (context, recorders) = build_context(&["clean"]);

        build_command(&["test/clean", "test"])
            .run(&context)
            .await
            .unwrap();

        assert!(!recorders.has_instance());
        assert_eq!(recorders.count_snapshot_deletes(), 0);
    }

    #[tokio::test]
    async fn test_reject_an_empty_target_list() {
        let (context, _) = build_context(&[]);

        assert!(matches!(
            build_command(&[]).run(&context).await,
            Err(Error::MissingInstanceName)
        ));
    }

    #[tokio::test]
    async fn test_reject_an_unknown_snapshot() {
        let (context, recorders) = build_context(&["deps"]);

        assert!(matches!(
            build_command(&["test/clean"]).run(&context).await,
            Err(Error::UnknownSnapshot(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
        assert!(recorders.has_instance());
    }
}
