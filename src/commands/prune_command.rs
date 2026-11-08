use crate::actions::{DeleteInstanceAction, ListInstancesAction, LoadInstanceAction};
use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::{DataSize, Instance};
use crate::util::ResourceLock;
use crate::view::ConfirmDialog;
use clap::Parser;
use std::ffi::OsStr;
use std::path::PathBuf;

const LEGACY_INSTANCES_DIR: &str = "instances";
const LEGACY_CERT_FILES: [&str; 5] = [
    "ca-cert.pem",
    "server-cert.pem",
    "server-key.pem",
    "client-cert.pem",
    "client-key.pem",
];

/// Remove stale files
///
/// This command removes stopped temporary instances and files left behind
/// by interrupted runs or older versions of cubic.
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
        let instance_names = ListInstancesAction::new().run(context);

        let stale_instances: Vec<Instance> = instance_names
            .iter()
            .filter_map(|name| LoadInstanceAction::new().read(context, name.as_str()).ok())
            .filter(|instance| context.is_stale(instance))
            .collect();
        let stale_dirs: Vec<PathBuf> = stale_instances
            .iter()
            .map(|instance| env.get_instance_dir2(&instance.name))
            .collect();
        // A create that was interrupted before its final rename leaves a .tmp dir
        let tmp_dirs: Vec<PathBuf> = system
            .read_dir(&env.get_instance_dir())
            .unwrap_or_default()
            .into_iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "tmp"))
            .collect();

        // An interrupted download leaves a .tmp file
        let downloads: Vec<PathBuf> = system
            .read_dir(&env.get_image_dir())
            .unwrap_or_default()
            .into_iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "tmp"))
            .collect();

        let legacy_certs: Vec<PathBuf> = instance_names
            .iter()
            .flat_map(|name| {
                let dir = env.get_instance_dir2(name.as_str());
                LEGACY_CERT_FILES.map(|file| dir.join(file))
            })
            .filter(|path| system.exists_path(path))
            .collect();

        let legacy_dir = env.get_cache_dir().join(LEGACY_INSTANCES_DIR);

        let cache_file = env.get_image_cache_file();
        if tmp_dirs.is_empty()
            && downloads.is_empty()
            && stale_dirs.is_empty()
            && legacy_certs.is_empty()
            && !system.exists_path(&cache_file)
            && !system.exists_path(&legacy_dir)
        {
            console.print("Nothing to prune");
            return Ok(0);
        }

        // Calculate size
        let bytes = tmp_dirs
            .iter()
            .chain(downloads.iter())
            .chain(stale_dirs.iter())
            .chain(legacy_certs.iter())
            .chain([&cache_file, &legacy_dir])
            .fold(0, |total, path| total + system.get_path_size(path));
        let total = DataSize::new(bytes as usize).to_size();

        // Print size of files to be deleted
        console.print(&format!("Pruning frees {total} of disk space.\n"));

        if self.yes.value
            || ConfirmDialog::new("Are you sure you want to continue?").confirm(console)
        {
            // Delete files
            let mut freed = system.get_path_size(&cache_file) + system.get_path_size(&legacy_dir);
            system.remove_file(&cache_file).ok();
            system.remove_dir(&legacy_dir).ok();
            for dir in &tmp_dirs {
                if let Some(name) = dir.file_stem().and_then(OsStr::to_str)
                    && let Ok(Some(_lock)) = ResourceLock::try_acquire_instance(context, name)
                {
                    freed += system.get_path_size(dir);
                    system.remove_dir(dir).ok();
                }
            }
            for file in &downloads {
                if let Some(name) = file.file_name().and_then(OsStr::to_str)
                    && let Ok(Some(_lock)) = ResourceLock::try_acquire_image(context, name)
                {
                    freed += system.get_path_size(file);
                    system.remove_file(file).ok();
                }
            }
            for cert in &legacy_certs {
                freed += system.get_path_size(cert);
                system.remove_file(cert).ok();
            }
            // Skip an instance that a session started while the dialog above
            // waited for an answer.
            for instance in &stale_instances {
                let lock = ResourceLock::try_acquire_instance(context, &instance.name);
                if let Ok(Some(_lock)) = lock
                    && context.is_stale(instance)
                {
                    freed += system.get_path_size(&env.get_instance_dir2(&instance.name));
                    DeleteInstanceAction::new().run(context, instance).ok();
                }
            }

            // Print size of deleted files
            let freed = DataSize::new(freed as usize).to_size();
            console.print(&format!("Successfully freed {freed} of disk space."));
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
    async fn test_delete_the_legacy_image_list_and_cache_instance_dir() {
        let env = build_env();
        let builder = build_context(
            SystemMock::new()
                .add_file(env.get_image_cache_file(), b"cache")
                .add_file("/cache/instances/test/user-data.img", b"seed"),
        );
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert!(!system.exists_path(&env.get_image_cache_file()));
        assert!(!system.exists_path(Path::new("/cache/instances")));
    }

    #[tokio::test]
    async fn test_keep_cached_images() {
        let env = build_env();
        let image = env.get_image_file("debian");
        let builder = build_context(SystemMock::new().add_file(&image, b"image"));
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert!(system.exists_path(&image));
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
        let builder = build_context(LEGACY_CERT_FILES.iter().fold(
            SystemMock::new().add_file(dir.join("machine.img"), b"disk"),
            |system, file| system.add_file(dir.join(file), b"pem"),
        ));
        builder.add_instances(vec![Instance {
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
        let builder = build_context(SystemMock::new().add_dir(&tmp));
        builder.add_instances(vec![
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
        builder.set_running(&["running"]);
        let system = Arc::clone(builder.get_system());

        run_prune(builder).await;

        assert!(!system.exists_path(&env.get_instance_dir2("stale")));
        assert!(system.exists_path(&env.get_instance_dir2("running")));
        assert!(!system.exists_path(&tmp));
    }

    #[tokio::test]
    async fn test_keep_a_create_that_another_command_runs() {
        let env = build_env();
        let tmp = env.get_instance_dir().join("busy.tmp");
        let builder = build_context(SystemMock::new().add_dir(&tmp));
        let system = Arc::clone(builder.get_system());
        let lock = env.get_instance_dir().join("busy.lock");
        let _lock = system.lock_file(&lock, false).unwrap();

        run_prune(builder).await;

        assert!(system.exists_path(&tmp));
    }

    #[tokio::test]
    async fn test_keep_a_download_that_another_command_runs() {
        let env = build_env();
        let busy = env.get_image_file("busy.tmp");
        let idle = env.get_image_file("idle.tmp");
        let builder = build_context(
            SystemMock::new()
                .add_file(&busy, b"part")
                .add_file(&idle, b"part"),
        );
        let system = Arc::clone(builder.get_system());
        let lock = env.get_image_dir().join("busy.lock");
        let _lock = system.lock_file(&lock, false).unwrap();

        run_prune(builder).await;

        assert!(system.exists_path(&busy));
        assert!(!system.exists_path(&idle));
    }

    #[tokio::test]
    async fn test_report_nothing_to_prune() {
        let builder = build_context(SystemMock::new());

        assert_eq!(run_prune(builder).await, "Nothing to prune\n");
    }

    #[tokio::test]
    async fn test_report_the_size_of_everything_it_deletes() {
        let env = build_env();
        let builder = build_context(
            SystemMock::new()
                .add_file(env.get_image_file("debian.tmp"), &[0; 1024])
                .add_file("/cache/instances/test/user-data.img", &[0; 1024]),
        );

        assert!(run_prune(builder).await.contains("frees 2048 B"));
    }
}
