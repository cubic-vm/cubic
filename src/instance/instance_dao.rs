use crate::error::{Error, Result};
use crate::instance::{
    InstanceDeserializer, InstanceSerializer, InstanceStore, TomlInstanceDeserializer,
    YamlInstanceDeserializer,
};
use crate::models::{DataSize, Environment, Instance, InstanceName};
use crate::platform::System;
use crate::qemu::QemuImg;
use crate::qemu::QemuMonitorClient;
use std::path::Path;
use std::rc::Rc;
use std::str;
use std::str::FromStr;

pub struct InstanceDao {
    pub env: Environment,
    system: Rc<dyn System>,
}

impl InstanceDao {
    pub fn new(system: Rc<dyn System>, env: &Environment) -> Result<Self> {
        system.create_writable_dir(Path::new(&env.get_instance_dir()))?;
        system.create_writable_dir(Path::new(env.get_cache_dir()))?;
        system.create_writable_dir(Path::new(env.get_runtime_dir()))?;

        Ok(InstanceDao {
            env: env.clone(),
            system,
        })
    }

    fn read_running_pid(&self, instance: &Instance) -> Option<u64> {
        let pid = self
            .system
            .read_file_to_string(Path::new(&self.env.get_qemu_pid_file(&instance.name)))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()?;

        if self.system.exists_process(pid) {
            Some(pid)
        } else {
            self.system
                .remove_file(Path::new(&self.env.get_qemu_pid_file(&instance.name)))
                .ok();
            None
        }
    }
}

impl InstanceStore for InstanceDao {
    fn get_instances(&self) -> Vec<String> {
        let mut instances: Vec<String> = self
            .system
            .read_dir(Path::new(&self.env.get_instance_dir()))
            .ok()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry.file_name())
                    .filter_map(|name| name.to_str())
                    .filter(|name| InstanceName::from_str(name).is_ok())
                    .map(|name| name.to_string())
                    .collect()
            })
            .unwrap_or_default();
        instances.sort_by_key(|name| name.to_lowercase());
        instances
    }

    fn exists(&self, name: &str) -> bool {
        self.system
            .exists_path(Path::new(&self.env.get_instance_dir2(name)))
    }

    fn load(&self, name: &str) -> Result<Instance> {
        if !self.exists(name) {
            return Err(Error::UnknownInstance(name.to_string()));
        }

        let yaml_path = &self.env.get_instance_yaml_config_file(name);
        let toml_path = &self.env.get_instance_toml_config_file(name);

        let from_yaml = !self.system.exists_path(Path::new(toml_path));
        let (path, deserializer): (&str, Box<dyn InstanceDeserializer>) = if from_yaml {
            (yaml_path, Box::new(YamlInstanceDeserializer::new()))
        } else {
            (toml_path, Box::new(TomlInstanceDeserializer::new()))
        };

        let instance = self
            .system
            .open_file(Path::new(path))
            .ok()
            .and_then(|mut file| deserializer.deserialize(name, &mut file))
            .map(|mut instance| {
                // migrate the deprecated yaml config to the toml format
                if from_yaml {
                    self.store(&instance).ok();
                }

                if let Some(info) =
                    QemuImg::new(self.system.as_ref()).get_image_info(&self.env, &instance)
                {
                    instance.disk_used = Some(DataSize::new(info.actual_size as usize));
                    instance.disk_capacity = DataSize::new(info.virtual_size as usize);
                }
                instance
            });

        Ok(match instance {
            Some(i) => i,
            None => Instance {
                name: name.to_string(),
                user: self.env.get_username().clone(),
                cpus: 1,
                mem: DataSize::from_str("1G").unwrap(),
                disk_capacity: DataSize::from_str("1G").unwrap(),
                ssh_port: self.system.bind_port()?,
                ..Instance::default()
            },
        })
    }

    fn store(&self, instance: &Instance) -> Result<()> {
        let file_name = self.env.get_instance_toml_config_file(&instance.name);
        let temp_file_name = format!("{file_name}.tmp");

        let mut file = self.system.create_file(Path::new(&temp_file_name))?;
        InstanceSerializer::new().serialize(instance, &mut file)?;
        self.system
            .rename_file(Path::new(&temp_file_name), Path::new(&file_name))?;

        // remove deprecated yaml file format
        self.system
            .remove_file(Path::new(
                &self.env.get_instance_yaml_config_file(&instance.name),
            ))
            .ok();

        Ok(())
    }

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<()> {
        if self.exists(new_name) {
            Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.system.rename_file(
                Path::new(&self.env.get_instance_dir2(&instance.name)),
                Path::new(&self.env.get_instance_dir2(new_name)),
            )?;
            instance.name = new_name.to_string();
            Ok(())
        }
    }

    fn resize(&self, instance: &mut Instance, size: u64) -> Result<()> {
        if self.is_running(instance) {
            Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else if instance.disk_capacity.get_bytes() >= size as usize {
            Err(Error::CannotShrinkDisk(instance.name.to_string()))
        } else {
            QemuImg::new(self.system.as_ref())
                .resize(&self.env.get_instance_image_file(&instance.name), size)?;
            instance.disk_capacity = DataSize::new(size as usize);
            Ok(())
        }
    }

    fn delete(&self, instance: &Instance) -> Result<()> {
        if self.is_running(instance) {
            Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.system
                .remove_dir(Path::new(
                    &self.env.get_instance_runtime_dir(&instance.name),
                ))
                .ok();
            self.system
                .remove_dir(Path::new(&self.env.get_instance_cache_dir(&instance.name)))
                .ok();

            self.system
                .remove_dir(Path::new(&self.env.get_instance_dir2(&instance.name)))
                .ok();
            Ok(())
        }
    }

    fn is_running(&self, instance: &Instance) -> bool {
        self.read_running_pid(instance).is_some()
    }

    fn get_pid(&self, instance: &Instance) -> Option<u64> {
        self.read_running_pid(instance)
    }

    fn kill(&self, instance: &Instance) -> Result<()> {
        let pid = self
            .get_pid(instance)
            .ok_or_else(|| Error::InstanceNotRunning(instance.name.clone()))?;

        // A process that died between the check above and the signal already
        // reached the goal of this call, so it counts as success. Only a live
        // process that refused to die is an error, and it keeps its pid file,
        // since dropping the file would orphan a QEMU process that no later
        // command could reach.
        let result = match self.system.kill_process(pid) {
            Err(Error::ProcessNotFound(_)) => Ok(()),
            result => result,
        };

        if result.is_ok() {
            self.system
                .remove_file(Path::new(&self.env.get_qemu_pid_file(&instance.name)))
                .ok();
        }
        result
    }

    fn get_monitor(&self, instance: &Instance) -> Result<QemuMonitorClient> {
        QemuMonitorClient::new(&self.env, instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserName;
    use crate::platform::SystemMock;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
            "/run".to_string(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_exists_true_when_instance_dir_present() {
        let system = SystemMock::new().add_dir("/data/machines/test");
        let dao = InstanceDao::new(Rc::new(system), &build_env()).unwrap();

        assert!(dao.exists("test"));
        assert!(!dao.exists("missing"));
    }

    #[test]
    fn test_store_then_load_round_trips_instance() {
        let system = SystemMock::new().add_dir("/data/machines/test");
        let dao = InstanceDao::new(Rc::new(system), &build_env()).unwrap();
        let instance = Instance {
            name: "test".to_string(),
            ..Instance::default()
        };

        dao.store(&instance).unwrap();
        let loaded = dao.load("test").unwrap();

        assert_eq!(loaded.name, instance.name);
    }

    #[test]
    fn test_is_running_true_when_pid_alive() {
        let env = build_env();
        let system = SystemMock::new()
            .add_file(&env.get_qemu_pid_file("test"), b"1234\n")
            .add_process(1234);
        let dao = InstanceDao::new(Rc::new(system), &env).unwrap();

        assert!(dao.is_running(&build_instance()));
        assert_eq!(dao.get_pid(&build_instance()), Some(1234));
    }

    #[test]
    fn test_is_running_false_and_removes_stale_pid_file() {
        let env = build_env();
        let system = Rc::new(SystemMock::new().add_file(&env.get_qemu_pid_file("test"), b"1234\n"));
        let dao = InstanceDao::new(Rc::clone(&system) as Rc<dyn System>, &env).unwrap();

        assert!(!dao.is_running(&build_instance()));
        assert!(!system.exists_path(Path::new(&env.get_qemu_pid_file("test"))));
    }

    #[test]
    fn test_kill_kills_pid_and_removes_pid_file() {
        let env = build_env();
        let system = Rc::new(
            SystemMock::new()
                .add_file(&env.get_qemu_pid_file("test"), b"1234\n")
                .add_process(1234),
        );
        let dao = InstanceDao::new(Rc::clone(&system) as Rc<dyn System>, &env).unwrap();

        dao.kill(&build_instance()).unwrap();

        assert_eq!(system.get_killed_processes(), vec![1234]);
        assert!(!system.exists_path(Path::new(&env.get_qemu_pid_file("test"))));
    }

    #[test]
    fn test_kill_succeeds_when_the_process_died_first() {
        let env = build_env();
        let system = Rc::new(
            SystemMock::new()
                .add_file(&env.get_qemu_pid_file("test"), b"1234\n")
                .add_vanishing_process(1234),
        );
        let dao = InstanceDao::new(Rc::clone(&system) as Rc<dyn System>, &env).unwrap();

        // The process is gone, which is what the call asked for, so losing the
        // race to whoever reaped it is not a failure.
        dao.kill(&build_instance()).unwrap();

        assert!(system.get_killed_processes().is_empty());
        assert!(!system.exists_path(Path::new(&env.get_qemu_pid_file("test"))));
    }

    #[test]
    fn test_kill_keeps_pid_file_when_the_kill_fails() {
        let env = build_env();
        let system = Rc::new(
            SystemMock::new()
                .add_file(&env.get_qemu_pid_file("test"), b"1234\n")
                .add_unkillable_process(1234),
        );
        let dao = InstanceDao::new(Rc::clone(&system) as Rc<dyn System>, &env).unwrap();

        assert!(matches!(
            dao.kill(&build_instance()),
            Err(Error::KillFailed(1234))
        ));
        // The pid file has to survive, otherwise the still running QEMU
        // process would be unreachable for every later command.
        assert!(system.exists_path(Path::new(&env.get_qemu_pid_file("test"))));
        assert!(dao.is_running(&build_instance()));
    }

    #[test]
    fn test_kill_errors_when_not_running() {
        let system = SystemMock::new();
        let dao = InstanceDao::new(Rc::new(system), &build_env()).unwrap();

        assert!(matches!(
            dao.kill(&build_instance()),
            Err(Error::InstanceNotRunning(name)) if name == "test"
        ));
    }

    #[test]
    fn test_get_instances_lists_sorted_valid_names() {
        let system = SystemMock::new()
            .add_dir("/data/machines/zebra")
            .add_dir("/data/machines/apple");
        let dao = InstanceDao::new(Rc::new(system), &build_env()).unwrap();

        assert_eq!(dao.get_instances(), vec!["apple", "zebra"]);
    }
}
