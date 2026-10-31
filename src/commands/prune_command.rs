use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::{DataSize, Instance};
use crate::view::ConfirmDialog;
use clap::Parser;
use std::path::PathBuf;

const LEGACY_INSTANCES_DIR: &str = "instances";
const LEGACY_CERT_FILES: [&str; 5] = [
    "ca-cert.pem",
    "server-cert.pem",
    "server-key.pem",
    "client-cert.pem",
    "client-key.pem",
];

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
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let console = context.get_console();
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
            .map(|instance| env.get_instance_dir2(&instance.name))
            .collect();
        // A create that was interrupted before its final rename leaves a .tmp dir
        let tmp_dirs = system
            .read_dir(&env.get_instance_dir())
            .unwrap_or_default()
            .into_iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "tmp"));

        let legacy_certs: Vec<PathBuf> = instance_store
            .get_instances()
            .into_iter()
            .flat_map(|name| {
                let dir = env.get_instance_dir2(&name);
                LEGACY_CERT_FILES.map(|file| dir.join(file))
            })
            .filter(|path| system.exists_path(path))
            .collect();

        let dirs: Vec<PathBuf> = [
            env.get_image_dir(),
            env.get_cache_dir().join(LEGACY_INSTANCES_DIR),
        ]
        .into_iter()
        .chain(tmp_dirs)
        .collect();

        // Calculate size
        let cache_file = env.get_image_cache_file();
        let total = DataSize::new(
            dirs.iter()
                .chain(stale_dirs.iter())
                .chain(legacy_certs.iter())
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
            for cert in &legacy_certs {
                system.remove_file(cert).ok();
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
    use crate::commands::ContextBuilder;
    use crate::models::{Environment, UserName};
    use crate::platform::{FileSystem, SystemMock};
    use std::path::Path;
    use std::str::FromStr;
    use std::sync::Arc;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".into(),
            "/cache".into(),
        )
    }

    fn build_context(system: SystemMock) -> ContextBuilder {
        let mut builder = ContextBuilder::new(system);
        *builder.get_env() = build_env();
        builder
    }

    async fn run_prune(builder: ContextBuilder) -> String {
        let system = Arc::clone(builder.get_system());
        PruneCommand {
            yes: commands::YesArg { value: true },
        }
        .run(&builder.build())
        .await
        .unwrap();
        system.get_output()
    }

    #[tokio::test]
    async fn test_delete_the_image_cache_and_the_legacy_cache_instance_dir() {
        let env = build_env();
        let builder = build_context(
            SystemMock::new()
                .add_file(env.get_image_cache_file(), b"cache")
                .add_file(env.get_image_file("debian"), b"image")
                .add_file("/cache/instances/test/user-data.img", b"seed"),
        );
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert!(!system.exists_path(&env.get_image_cache_file()));
        assert!(!system.exists_path(&env.get_image_dir()));
        assert!(!system.exists_path(Path::new("/cache/instances")));
    }

    #[tokio::test]
    async fn test_keep_the_instance_data_dir() {
        let env = build_env();
        let instance_file = env.get_cloud_init_file("test");
        let builder = build_context(SystemMock::new().add_file(&instance_file, b"seed"));
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert!(system.exists_path(&instance_file));
    }

    #[tokio::test]
    async fn test_delete_the_legacy_certs_of_a_stopped_instance() {
        let env = build_env();
        let dir = env.get_instance_dir2("test");
        let mut builder = build_context(LEGACY_CERT_FILES.iter().fold(
            SystemMock::new().add_file(dir.join("machine.img"), b"disk"),
            |system, file| system.add_file(dir.join(file), b"pem"),
        ));
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            ..Instance::default()
        }]);
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        for file in LEGACY_CERT_FILES {
            assert!(!system.exists_path(&dir.join(file)));
        }
        assert!(system.exists_path(&dir.join("machine.img")));
    }

    #[tokio::test]
    async fn test_delete_stale_instances_and_interrupted_creates() {
        let env = build_env();
        let tmp = env.get_instance_dir().join("plain.tmp");
        let mut builder = build_context(SystemMock::new().add_dir(&tmp));
        builder.get_store().add_instances(vec![
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
        ]);
        builder.get_store().set_running(&["running"]);
        let deleted = Arc::clone(&builder.get_store().deleted);
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert_eq!(*deleted.lock().unwrap(), ["stale"]);
        assert!(!system.exists_path(&tmp));
    }

    #[tokio::test]
    async fn test_report_the_size_of_everything_it_deletes() {
        let env = build_env();
        let builder = build_context(
            SystemMock::new()
                .add_file(env.get_image_file("debian"), &[0; 1024])
                .add_file("/cache/instances/test/user-data.img", &[0; 1024]),
        );

        assert!(run_prune(builder).await.contains("frees 2048 B"));
    }
}
