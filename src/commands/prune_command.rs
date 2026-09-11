use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::{DataSize, Instance};
use crate::view::{ConfirmDialog, Console};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const LEGACY_INSTANCES_DIR: &str = "instances";

/// Clear caches
///
/// This command removes cached VM image files, stopped temporary instances
/// and instance files left behind by interrupted runs or older versions of
/// cubic.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct PruneCommand {
    #[clap(flatten)]
    yes: commands::YesArg,
}

impl Command for PruneCommand {
    async fn run(&self, console: &Arc<Console>, context: &commands::Context) -> Result<u8> {
        let env = context.get_env();
        let system = context.get_system();
        let instance_store = context.get_instance_store();

        let stale_instances: Vec<Instance> = instance_store
            .get_instances()
            .into_iter()
            .filter_map(|name| instance_store.load(&name).ok())
            .filter(|instance| instance_store.is_stale(instance))
            .collect();
        let stale_dirs: Vec<PathBuf> = stale_instances
            .iter()
            .map(|instance| PathBuf::from(env.get_instance_dir2(&instance.name)))
            .collect();
        // A create that was interrupted before its final rename leaves a .tmp dir
        let tmp_dirs = system
            .read_dir(Path::new(&env.get_instance_dir()))
            .unwrap_or_default()
            .into_iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "tmp"));

        let dirs: Vec<PathBuf> = [
            PathBuf::from(env.get_image_dir()),
            PathBuf::from(env.get_cache_dir()).join(LEGACY_INSTANCES_DIR),
        ]
        .into_iter()
        .chain(tmp_dirs)
        .collect();

        // Calculate size
        let cache_file = PathBuf::from(env.get_image_cache_file());
        let total = DataSize::new(
            dirs.iter()
                .chain(stale_dirs.iter())
                .chain([&cache_file])
                .fold(0, |total, path| total + system.get_path_size(path)) as usize,
        )
        .to_size();

        // Print size of files to be deleted
        console.print(&format!("Pruning caches frees {total} of disk space.\n"));

        if self.yes.value
            || ConfirmDialog::new("Are you sure you want to continue?").confirm(console)
        {
            // Delete files
            system.remove_file(&cache_file).ok();
            for dir in &dirs {
                system.remove_dir(dir).ok();
            }
            // The store skips an instance that a session started while the
            // dialog above waited for an answer.
            for instance in &stale_instances {
                instance_store.delete(instance).ok();
            }

            // Print size of deleted files
            console.print(&format!("Successfully freed {total} of disk space."));
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, UserName};
    use crate::platform::{FileSystem, System, SystemMock};
    use std::path::Path;
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
        )
    }

    fn build_context(system: &Arc<SystemMock>, env: &Environment) -> commands::Context {
        build_context_with_store(system, env, InstanceStoreMock::new(Vec::new()))
    }

    fn build_context_with_store(
        system: &Arc<SystemMock>,
        env: &Environment,
        store: InstanceStoreMock,
    ) -> commands::Context {
        commands::Context::new(
            Arc::clone(system) as Arc<dyn System>,
            env.clone(),
            Box::new(store),
        )
    }

    async fn run_prune(system: &Arc<SystemMock>, env: &Environment) -> String {
        run_prune_with_context(system, build_context(system, env)).await
    }

    async fn run_prune_with_context(
        system: &Arc<SystemMock>,
        context: commands::Context,
    ) -> String {
        let console = &Console::new(Arc::clone(system) as Arc<dyn System>);
        PruneCommand {
            yes: commands::YesArg { value: true },
        }
        .run(console, &context)
        .await
        .unwrap();
        system.get_output()
    }

    #[tokio::test]
    async fn test_delete_the_image_cache_and_the_legacy_cache_instance_dir() {
        let env = build_env();
        let system = Arc::new(
            SystemMock::new()
                .add_file(&env.get_image_cache_file(), b"cache")
                .add_file(&format!("{}/debian", env.get_image_dir()), b"image")
                .add_file("/cache/instances/test/user-data.img", b"seed"),
        );

        run_prune(&system, &env).await;

        assert!(!system.exists_path(Path::new(&env.get_image_cache_file())));
        assert!(!system.exists_path(Path::new(&env.get_image_dir())));
        assert!(!system.exists_path(Path::new("/cache/instances")));
    }

    #[tokio::test]
    async fn test_keep_the_instance_data_dir() {
        let env = build_env();
        let instance_file = format!("{}/cloud-init.iso", env.get_instance_dir2("test"));
        let system = Arc::new(SystemMock::new().add_file(&instance_file, b"seed"));

        run_prune(&system, &env).await;

        assert!(system.exists_path(Path::new(&instance_file)));
    }

    #[tokio::test]
    async fn test_delete_stale_instances_and_interrupted_creates() {
        let env = build_env();
        let tmp = format!("{}/plain.tmp", env.get_instance_dir());
        let system = Arc::new(SystemMock::new().add_dir(&tmp));
        let store = InstanceStoreMock::new_with_running(
            vec![
                Instance {
                    name: "stale".to_string(),
                    auto_remove: true,
                    ..Instance::default()
                },
                Instance {
                    name: "running".to_string(),
                    auto_remove: true,
                    ..Instance::default()
                },
                Instance {
                    name: "plain".to_string(),
                    ..Instance::default()
                },
            ],
            &["running"],
        );
        let deleted = Arc::clone(&store.deleted);

        run_prune_with_context(&system, build_context_with_store(&system, &env, store)).await;

        assert_eq!(*deleted.lock().unwrap(), ["stale"]);
        assert!(!system.exists_path(Path::new(&tmp)));
    }

    #[tokio::test]
    async fn test_report_the_size_of_everything_it_deletes() {
        let env = build_env();
        let system = Arc::new(
            SystemMock::new()
                .add_file(&format!("{}/debian", env.get_image_dir()), &[0; 1024])
                .add_file("/cache/instances/test/user-data.img", &[0; 1024]),
        );

        assert!(run_prune(&system, &env).await.contains("frees 2048 B"));
    }
}
