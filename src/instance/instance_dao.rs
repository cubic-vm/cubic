use crate::error::{Error, Result};
use crate::instance::{
    InstanceDeserializer, InstanceSerializer, InstanceStore, TomlInstanceDeserializer,
    YamlInstanceDeserializer,
};
use crate::models::{DataSize, Environment, Instance, InstanceName};
use crate::platform::System;
use crate::qemu::Monitor;
use crate::qemu::QemuImg;
use crate::ssh_cmd::PortChecker;
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

    fn is_process_alive(&self, pid: u64) -> bool {
        let sys_pid = sysinfo::Pid::from_u32(pid as u32);
        let mut system = sysinfo::System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);
        system.process(sys_pid).is_some()
    }

    fn read_running_pid(&self, instance: &Instance) -> Option<u64> {
        let pid = self
            .system
            .read_file_to_string(Path::new(&self.env.get_qemu_pid_file(&instance.name)))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()?;

        if self.is_process_alive(pid) {
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
                ssh_port: PortChecker::new().get_new_port()?,
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

        let sys_pid = sysinfo::Pid::from_u32(pid as u32);
        let mut system = sysinfo::System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);
        if let Some(process) = system.process(sys_pid) {
            process.kill();
        }

        self.system
            .remove_file(Path::new(&self.env.get_qemu_pid_file(&instance.name)))
            .ok();
        Ok(())
    }

    fn get_monitor(&self, instance: &Instance) -> Result<Monitor> {
        Monitor::new(&self.env, instance)
    }
}
