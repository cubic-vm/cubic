use crate::actions::{DeleteInstanceAction, LoadInstanceAction};
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::image::{ImageFactory, ImageStore};
use crate::models::{ImageName, InstanceName, ResourceName};
use crate::qemu::QemuImg;
use crate::util::ResourceLock;
use crate::view::ConfirmDialog;
use clap::Parser;
use std::collections::HashSet;

/// Delete VM instances, snapshots and images
///
/// A target written as <INSTANCE> deletes the whole VM instance including all
/// of its snapshots. A target written as <INSTANCE>/<SNAPSHOT> deletes only
/// that one snapshot and leaves the VM instance alone. A target written as
/// <DISTRO>:<TAG> deletes the cached VM image file.
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
///   Delete the cached VM image of the latest Ubuntu release:
///   $ cubic delete ubuntu:latest
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
    /// Names of the virtual machine instances, their snapshots or images
    #[clap(value_name = "TARGETS", required = true)]
    targets: Vec<ResourceName>,
}

impl DeleteCommand {
    /// The targets to act on. A snapshot whose instance is deleted as a whole is
    /// dropped, because deleting the instance already removes its snapshots.
    /// Duplicates are dropped so a target is never deleted twice.
    fn get_targets(&self) -> Vec<&ResourceName> {
        let deleted_instances: HashSet<&InstanceName> = self
            .targets
            .iter()
            .filter_map(|target| match target {
                ResourceName::Instance(instance) => Some(instance),
                _ => None,
            })
            .collect();

        let mut seen = HashSet::new();
        self.targets
            .iter()
            .filter(|target| match target {
                ResourceName::Snapshot(snapshot) => {
                    !deleted_instances.contains(snapshot.get_instance())
                }
                _ => true,
            })
            .filter(|target| seen.insert(target.to_string()))
            .collect()
    }

    async fn delete_target(context: &commands::Context, target: &ResourceName) -> Result<()> {
        let instance_name = match target {
            ResourceName::Instance(instance) => instance,
            ResourceName::Snapshot(snapshot) => snapshot.get_instance(),
            ResourceName::Image(name) => return Self::delete_image(context, name),
        };
        let _lock = ResourceLock::acquire_instances(context, &[instance_name.as_str()])?;

        // Free the disk lock first. A full instance delete can kill it, a
        // snapshot needs the instance shut down cleanly.
        commands::StopCommand {
            all: false.into(),
            wait: true,
            kill: matches!(target, ResourceName::Instance(_)),
            instances: vec![instance_name.clone()].into(),
        }
        .run(context)
        .await?;

        let instance = LoadInstanceAction::new().run(context, instance_name.as_str())?;
        match target {
            ResourceName::Snapshot(snapshot_name) => {
                let image = context.get_env().get_instance_image_file(&instance.name);
                QemuImg::new(context.get_system()).delete_snapshot(&image, snapshot_name.as_str())
            }
            _ => DeleteInstanceAction::new().run(context, &instance),
        }
    }

    fn check_image(context: &commands::Context, name: &ImageName) -> Result<()> {
        let image = ImageFactory::find_image(name)?;
        if !ImageStore::new().exists(context.get_system(), context.get_env(), &image) {
            return Err(Error::NotFound(name.to_string()));
        }
        Ok(())
    }

    fn delete_image(context: &commands::Context, name: &ImageName) -> Result<()> {
        let image = ImageFactory::find_image(name)?;
        let _lock = ResourceLock::acquire_image(context, &image)?;
        // Two tags such as ubuntu:latest and ubuntu:resolute share one file
        let file = context.get_env().get_image_file(&image.to_file_name());
        if context.get_system().exists_path(&file) {
            context.get_system().remove_file(&file)?;
        }
        Ok(())
    }
}

impl Command for DeleteCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();

        for target in &self.targets {
            let instance_name = match target {
                ResourceName::Instance(instance) => instance,
                ResourceName::Snapshot(snapshot) => snapshot.get_instance(),
                ResourceName::Image(name) => {
                    Self::check_image(context, name)?;
                    continue;
                }
            };
            if !context.has_instance(instance_name.as_str()) {
                return Err(Error::NotFound(instance_name.to_string()));
            }

            if let ResourceName::Snapshot(snapshot_name) = target {
                let env = context.get_env();
                let image = env.get_instance_image_file(instance_name.as_str());
                let qemu_img = QemuImg::new(context.get_system());
                if !qemu_img.has_snapshot(&image, snapshot_name.as_str()) {
                    return Err(Error::NotFound(target.to_string()));
                }
            }
        }

        let targets = self.get_targets();

        let is_image = |target: &&ResourceName| matches!(target, ResourceName::Image(_));
        let only_images = targets.iter().all(is_image);
        if !only_images {
            console.info("Running instances are stopped before deletion.");
        }
        if !self.yes.value && !ConfirmDialog::new("Do you want to proceed?").confirm(console) {
            return Ok(0);
        }

        let mut failed = false;
        for target in &targets {
            match Self::delete_target(context, target).await {
                Ok(()) => console.debug(&format!("Deleted {target}")),
                Err(error) => {
                    console.error(&format!("Failed to delete {target}: {error}"));
                    failed = true;
                }
            }
        }

        if failed {
            return Ok(1);
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
    async fn test_delete_continues_after_a_failed_target() {
        // Deleting test/deps fails because qemu-img has no recorded answer for it
        let (context, recorders) = build_context(&["deps", "clean"]);

        let command = build_command(&["test/deps", "test/clean"]);

        assert!(matches!(command.run(&context).await, Ok(1)));
        assert_eq!(recorders.count_snapshot_deletes(), 1);
    }

    #[tokio::test]
    async fn test_reject_an_unknown_snapshot() {
        let (context, recorders) = build_context(&["deps"]);

        assert!(matches!(
            build_command(&["test/clean"]).run(&context).await,
            Err(Error::NotFound(name)) if name == "test/clean"
        ));
        assert!(recorders.has_instance());
    }

    #[tokio::test]
    async fn test_delete_image_removes_the_cached_file() {
        let file = Environment::default().get_image_file("debian_bookworm_amd64");
        let builder = ContextBuilder::new(SystemMock::new().add_file(&file, b"image"));
        let system = Arc::clone(builder.get_system());

        build_command(&["debian:bookworm:amd64"])
            .run(&builder.build())
            .await
            .unwrap();

        assert!(!system.exists_path(&file));
    }

    #[tokio::test]
    async fn test_reject_an_uncached_image() {
        let (context, _) = build_context(&[]);
        let image: ImageName = "debian:bookworm:amd64".parse().unwrap();

        assert!(matches!(
            build_command(&["debian:bookworm:amd64"]).run(&context).await,
            Err(Error::NotFound(name)) if name == image.to_string()
        ));
    }
}
