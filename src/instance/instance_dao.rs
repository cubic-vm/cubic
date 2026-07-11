use crate::error::{Error, Result};
use crate::fs::FS;
use crate::instance::{
    InstanceDeserializer, InstanceSerializer, InstanceStore, TomlInstanceDeserializer,
    YamlInstanceDeserializer,
};
use crate::models::{DataSize, Environment, Instance, InstanceName};
use crate::qemu::Monitor;
use crate::qemu::QemuImg;
use crate::ssh_cmd::PortChecker;
use std::fs::DirEntry;
use std::path::Path;
use std::str;
use std::str::FromStr;

pub struct InstanceDao {
    fs: FS,
    pub env: Environment,
}

impl InstanceDao {
    pub fn new(env: &Environment) -> Result<Self> {
        let fs = FS::new();
        fs.setup_directory_access(&env.get_instance_dir())?;
        fs.setup_directory_access(env.get_cache_dir())?;
        fs.setup_directory_access(env.get_runtime_dir())?;

        Ok(InstanceDao {
            fs,
            env: env.clone(),
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
            .fs
            .read_file_to_string(&self.env.get_qemu_pid_file(&instance.name))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()?;

        if self.is_process_alive(pid) {
            Some(pid)
        } else {
            self.fs
                .remove_file(&self.env.get_qemu_pid_file(&instance.name))
                .ok();
            None
        }
    }
}

impl InstanceStore for InstanceDao {
    fn get_instances(&self) -> Vec<String> {
        let mut instances: Vec<String> = self
            .fs
            .read_dir(&self.env.get_instance_dir())
            .map_err(|_| ())
            .and_then(|entries| {
                entries
                    .collect::<std::result::Result<Vec<DirEntry>, _>>()
                    .map_err(|_| ())
            })
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry.file_name().to_str().map(|x| x.to_string()))
                    .filter(|entry| InstanceName::from_str(entry).is_ok())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        instances.sort_by_key(|a| a.to_lowercase());
        instances
    }

    fn exists(&self, name: &str) -> bool {
        Path::new(&self.env.get_instance_dir2(name)).exists()
    }

    fn load(&self, name: &str) -> Result<Instance> {
        if !self.exists(name) {
            return Err(Error::UnknownInstance(name.to_string()));
        }

        let yaml_path = &self.env.get_instance_yaml_config_file(name);
        let toml_path = &self.env.get_instance_toml_config_file(name);

        let from_yaml = !Path::new(toml_path).exists();
        let (path, deserializer): (&str, Box<dyn InstanceDeserializer>) = if from_yaml {
            (yaml_path, Box::new(YamlInstanceDeserializer::new()))
        } else {
            (toml_path, Box::new(TomlInstanceDeserializer::new()))
        };

        let instance = self
            .fs
            .open_file(path)
            .ok()
            .and_then(|mut file| deserializer.deserialize(name, &mut file))
            .map(|mut instance| {
                // migrate the deprecated yaml config to the toml format
                if from_yaml {
                    self.store(&instance).ok();
                }

                if let Some(info) = QemuImg::new().get_image_info(&self.env, &instance) {
                    instance.disk_used = Some(DataSize::new(info.actual_size as usize));
                    instance.disk_capacity = DataSize::new(info.virtual_size as usize);
                }
                instance
            });

        Ok(match instance {
            Some(i) => i,
            None => Instance {
                name: name.to_string(),
                user: self.env.get_username().to_string(),
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

        let mut file = self.fs.create_file(&temp_file_name)?;
        InstanceSerializer::new().serialize(instance, &mut file)?;
        self.fs.rename_file(&temp_file_name, &file_name)?;

        // remove deprecated yaml file format
        self.fs
            .remove_file(&self.env.get_instance_yaml_config_file(&instance.name))
            .ok();

        Ok(())
    }

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<()> {
        if self.exists(new_name) {
            Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs.rename_file(
                &self.env.get_instance_dir2(&instance.name),
                &self.env.get_instance_dir2(new_name),
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
            QemuImg::new().resize(&self.env.get_instance_image_file(&instance.name), size)?;
            instance.disk_capacity = DataSize::new(size as usize);
            Ok(())
        }
    }

    fn delete(&self, instance: &Instance) -> Result<()> {
        if self.is_running(instance) {
            Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs
                .remove_dir(&self.env.get_instance_runtime_dir(&instance.name))
                .ok();
            self.fs
                .remove_dir(&self.env.get_instance_cache_dir(&instance.name))
                .ok();

            self.fs
                .remove_dir(&self.env.get_instance_dir2(&instance.name))
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

        self.fs
            .remove_file(&self.env.get_qemu_pid_file(&instance.name))
            .ok();
        Ok(())
    }

    fn get_monitor(&self, instance: &Instance) -> Result<Monitor> {
        Monitor::new(&self.env, instance)
    }
}
