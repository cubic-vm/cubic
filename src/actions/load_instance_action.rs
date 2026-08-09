use crate::commands::{Context, DEFAULT_DISK_SIZE};
use crate::error::{Error, Result};
use crate::models::{DataSize, Instance, ResourceAllocator, get_default_arch};
use crate::qemu::QemuImg;
use crate::view::Console;
use std::str::FromStr;

/// Load an instance and replace an unreadable config with a machine of the
/// same defaults `cubic create` picks.
#[derive(Default)]
pub struct LoadInstanceAction;

impl LoadInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &self,
        context: &Context,
        console: &mut Console<'_>,
        name: &str,
    ) -> Result<Instance> {
        match context.get_instance_store().load(name) {
            Err(Error::InvalidInstanceConfig(_, reason)) => {
                console.warn(&format!(
                    "Config of instance '{name}' is invalid, resetting it to the default settings.\n{reason}"
                ));

                let instance = self.build_default_instance(context, name)?;
                context.get_instance_store().store(&instance)?;
                Ok(instance)
            }
            other => other,
        }
    }

    fn build_default_instance(&self, context: &Context, name: &str) -> Result<Instance> {
        let (cpus, mem) =
            ResourceAllocator::read_from_host(context.get_system()).get_default_resources();

        let mut instance = Instance {
            name: name.to_string(),
            arch: get_default_arch(),
            user: context.get_env().get_username().clone(),
            cpus,
            mem,
            disk_capacity: DataSize::from_str(DEFAULT_DISK_SIZE).unwrap(),
            ssh_port: context.get_system().bind_port()?,
            ..Instance::default()
        };

        QemuImg::new(context.get_system()).read_disk_info(context.get_env(), &mut instance);

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{InstanceDao, InstanceStore};
    use crate::models::{Environment, UserName};
    use crate::platform::{FileSystem, System, SystemMock};
    use std::path::Path;
    use std::rc::Rc;

    const GIB: usize = 1024 * 1024 * 1024;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
            "/run".to_string(),
        )
    }

    fn build_dao(system: &Rc<SystemMock>) -> InstanceDao {
        InstanceDao::new(Rc::clone(system) as Rc<dyn System>, &build_env()).unwrap()
    }

    fn build_context(system: &Rc<SystemMock>) -> Context {
        Context::new(
            Rc::clone(system) as Rc<dyn System>,
            build_env(),
            Box::new(build_dao(system)),
        )
    }

    fn build_system(config: &[u8]) -> Rc<SystemMock> {
        Rc::new(
            SystemMock::new()
                .set_host_resources((16 * GIB) as u64, (16 * GIB) as u64, 8)
                .add_dir("/data/machines/test")
                .add_file(&build_env().get_instance_toml_config_file("test"), config),
        )
    }

    #[test]
    fn test_run_replaces_a_broken_config_with_the_default_machine() {
        let system = build_system(b"cpus = ");
        let context = build_context(&system);
        let console = &mut Console::new(system.as_ref());

        let instance = LoadInstanceAction::new()
            .run(&context, console, "test")
            .unwrap();

        let (cpus, mem) = ResourceAllocator::new(16 * GIB, 8).get_default_resources();
        assert_eq!(instance.cpus, cpus);
        assert_eq!(instance.mem, mem);
        assert_eq!(
            instance.disk_capacity,
            DataSize::from_str(DEFAULT_DISK_SIZE).unwrap()
        );
        assert_eq!(instance.user.as_str(), "cubic");
        assert!(instance.ssh_port > 0);
    }

    #[test]
    fn test_run_writes_the_default_machine_back() {
        let system = build_system(b"cpus = ");
        let context = build_context(&system);
        let console = &mut Console::new(system.as_ref());

        let instance = LoadInstanceAction::new()
            .run(&context, console, "test")
            .unwrap();
        let reloaded = build_dao(&system).load("test").unwrap();

        assert_eq!(reloaded.cpus, instance.cpus);
        assert_eq!(reloaded.mem, instance.mem);
        assert_eq!(reloaded.ssh_port, instance.ssh_port);
    }

    #[test]
    fn test_run_keeps_a_valid_config() {
        let config = r#"
cpus = 3
mem = 1073741824
disk_capacity = 2361393152
ssh_port = 14357
"#;
        let system = build_system(config.as_bytes());
        let context = build_context(&system);
        let console = &mut Console::new(system.as_ref());

        let instance = LoadInstanceAction::new()
            .run(&context, console, "test")
            .unwrap();

        assert_eq!(instance.cpus, 3);
        assert_eq!(instance.ssh_port, 14357);
        assert_eq!(
            system
                .read_file_to_string(Path::new(
                    &build_env().get_instance_toml_config_file("test")
                ))
                .unwrap(),
            config
        );
    }

    #[test]
    fn test_run_passes_an_unknown_instance_on() {
        let system = Rc::new(SystemMock::new());
        let context = build_context(&system);
        let console = &mut Console::new(system.as_ref());

        assert!(matches!(
            LoadInstanceAction::new().run(&context, console, "missing"),
            Err(Error::UnknownInstance(name)) if name == "missing"
        ));
    }
}
