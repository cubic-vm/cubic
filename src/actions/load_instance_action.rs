use crate::actions::StoreInstanceAction;
use crate::commands::Context;
use crate::error::{Error, Result};
use crate::instance::TomlInstanceDeserializer;
use crate::models::{Arch, Instance, ResourceAllocator};

/// Load an instance and replace an unreadable config with a machine of the
/// same defaults `cubic create` picks.
#[derive(Default)]
pub struct LoadInstanceAction;

impl LoadInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, context: &Context, name: &str) -> Result<Instance> {
        match self.read(context, name) {
            Err(error @ Error::InvalidInstanceConfig { .. }) => {
                context.get_console().warn(&format!(
                    "{error}\n\nResetting instance '{name}' to the default settings."
                ));

                let instance = self.build_default_instance(context, name)?;
                StoreInstanceAction::new().run(context, &instance)?;
                Ok(instance)
            }
            other => other,
        }
    }

    /// Reads the config as it is, without replacing a broken one
    pub fn read(&self, context: &Context, name: &str) -> Result<Instance> {
        if !context.has_instance(name) {
            return Err(Error::UnknownInstance(name.to_string()));
        }

        let path = context.get_env().get_instance_toml_config_file(name);
        let mut file = context
            .get_system()
            .open_file(&path)
            .map_err(|error| Error::from_config(name, error))?;
        TomlInstanceDeserializer::new().deserialize(name, &mut file)
    }

    fn build_default_instance(&self, context: &Context, name: &str) -> Result<Instance> {
        let (cpus, mem) =
            ResourceAllocator::read_from_host(context.get_system()).get_default_resources();

        Ok(Instance {
            name: name.to_string(),
            arch: Arch::get_host(),
            user: context.get_env().get_username().clone(),
            cpus,
            mem,
            ssh_port: context.get_system().bind_port()?,
            ..Instance::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Environment, UserName};
    use crate::platform::SystemMock;
    use crate::view::Console;
    use std::str::FromStr;
    use std::sync::Arc;

    const GIB: usize = 1024 * 1024 * 1024;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".into(),
            "/cache".into(),
        )
    }

    fn build_context(system: &Arc<SystemMock>) -> Context {
        Context::new(system.clone(), Console::new(system.clone()), build_env())
    }

    fn build_system(config: &[u8]) -> Arc<SystemMock> {
        Arc::new(
            SystemMock::new()
                .set_host_resources((16 * GIB) as u64, (16 * GIB) as u64, 8)
                .add_dir("/data/machines/test")
                .add_file(build_env().get_instance_toml_config_file("test"), config),
        )
    }

    #[test]
    fn test_run_replaces_a_broken_config_with_the_default_machine() {
        let system = build_system(b"cpus = ");
        let context = build_context(&system);

        let instance = LoadInstanceAction::new().run(&context, "test").unwrap();

        let (cpus, mem) = ResourceAllocator::new(16 * GIB, 8).get_default_resources();
        assert_eq!(instance.cpus, cpus);
        assert_eq!(instance.mem, mem);
        assert_eq!(instance.user.as_str(), "cubic");
        assert!(instance.ssh_port > 0);
    }

    #[test]
    fn test_run_writes_the_default_machine_back() {
        let system = build_system(b"cpus = ");
        let context = build_context(&system);

        let instance = LoadInstanceAction::new().run(&context, "test").unwrap();
        let reloaded = LoadInstanceAction::new().read(&context, "test").unwrap();

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

        let instance = LoadInstanceAction::new().run(&context, "test").unwrap();

        assert_eq!(instance.cpus, 3);
        assert_eq!(instance.ssh_port, 14357);
        let written = system.get_written_file(build_env().get_instance_toml_config_file("test"));
        assert_eq!(written, Some(config.as_bytes().to_vec()));
    }

    #[test]
    fn test_run_passes_an_unknown_instance_on() {
        let system = Arc::new(SystemMock::new());
        let context = build_context(&system);

        assert!(matches!(
            LoadInstanceAction::new().run(&context, "missing"),
            Err(Error::UnknownInstance(name)) if name == "missing"
        ));
    }

    #[test]
    fn test_read_reports_a_missing_config() {
        let system = Arc::new(SystemMock::new().add_dir("/data/machines/test"));
        let context = build_context(&system);

        assert!(matches!(
            LoadInstanceAction::new().read(&context, "test"),
            Err(Error::InvalidInstanceConfig { name, .. }) if name == "test"
        ));
    }
}
