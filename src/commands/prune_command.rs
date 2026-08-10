use crate::commands::{self, Command};
use crate::error::Result;
use crate::models::DataSize;
use crate::view::{ConfirmDialog, Console};
use clap::Parser;
use std::path::PathBuf;

const LEGACY_INSTANCES_DIR: &str = "instances";

/// Clear caches
///
/// This command removes cached VM image files and instance files left behind
/// by older versions of cubic.
///
#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct PruneCommand {
    #[clap(flatten)]
    yes: commands::YesArg,
}

impl Command for PruneCommand {
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let system = context.get_system();

        let dirs = [
            PathBuf::from(env.get_image_dir()),
            PathBuf::from(env.get_cache_dir()).join(LEGACY_INSTANCES_DIR),
        ];

        // Calculate size
        let cache_file = PathBuf::from(env.get_image_cache_file());
        let total = DataSize::new(
            dirs.iter()
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

            // Print size of deleted files
            console.print(&format!("Successfully freed {total} of disk space."));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Environment, UserName};
    use crate::platform::{FileSystem, System, SystemMock};
    use std::path::Path;
    use std::rc::Rc;
    use std::str::FromStr;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
        )
    }

    fn build_context(system: &Rc<SystemMock>, env: &Environment) -> commands::Context {
        commands::Context::new(
            Rc::clone(system) as Rc<dyn System>,
            env.clone(),
            Box::new(InstanceStoreMock::new(Vec::new())),
        )
    }

    fn run_prune(system: &Rc<SystemMock>, env: &Environment) -> String {
        let console = &mut Console::new(system.as_ref());
        PruneCommand {
            yes: commands::YesArg { value: true },
        }
        .run(console, &build_context(system, env))
        .unwrap();
        system.get_output()
    }

    #[test]
    fn test_delete_the_image_cache_and_the_legacy_cache_instance_dir() {
        let env = build_env();
        let system = Rc::new(
            SystemMock::new()
                .add_file(&env.get_image_cache_file(), b"cache")
                .add_file(&format!("{}/debian", env.get_image_dir()), b"image")
                .add_file("/cache/instances/test/user-data.img", b"seed"),
        );

        run_prune(&system, &env);

        assert!(!system.exists_path(Path::new(&env.get_image_cache_file())));
        assert!(!system.exists_path(Path::new(&env.get_image_dir())));
        assert!(!system.exists_path(Path::new("/cache/instances")));
    }

    #[test]
    fn test_keep_the_instance_data_dir() {
        let env = build_env();
        let instance_file = format!("{}/cloud-init.iso", env.get_instance_dir2("test"));
        let system = Rc::new(SystemMock::new().add_file(&instance_file, b"seed"));

        run_prune(&system, &env);

        assert!(system.exists_path(Path::new(&instance_file)));
    }

    #[test]
    fn test_report_the_size_of_everything_it_deletes() {
        let env = build_env();
        let system = Rc::new(
            SystemMock::new()
                .add_file(&format!("{}/debian", env.get_image_dir()), &[0; 1024])
                .add_file("/cache/instances/test/user-data.img", &[0; 1024]),
        );

        assert!(run_prune(&system, &env).contains("frees 2.0 KiB"));
    }
}
