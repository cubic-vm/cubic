use crate::error::Error;
use crate::fs::FS;
use crate::instance::{Instance, InstanceState, InstanceStore};
use crate::qemu::Monitor;
use crate::ssh_cmd::PortChecker;
use crate::util;
use crate::util::SystemCommand;
use serde::{Deserialize, Serialize};
use std::fs::DirEntry;
use std::path::Path;
use std::str;

pub const USER: &str = "cubic";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: Instance,
}

pub struct InstanceDao {
    fs: FS,
    pub instance_dir: String,
    pub cache_dir: String,
}

impl InstanceDao {
    pub fn new() -> Result<Self, Error> {
        let fs = FS::new();
        let instance_dir = util::get_instance_data_dir()?;
        let xdg_runtime_dir = util::get_xdg_runtime_dir()?;
        let cache_dir = format!("{xdg_runtime_dir}/cubic/instances");
        fs.setup_directory_access(&instance_dir)?;
        fs.setup_directory_access(&cache_dir)?;

        Result::Ok(InstanceDao {
            fs,
            instance_dir,
            cache_dir,
        })
    }
}

impl InstanceStore for InstanceDao {
    fn get_instances(&self) -> Vec<String> {
        self.fs
            .read_dir(&self.instance_dir)
            .map_err(|_| ())
            .and_then(|entries| {
                entries
                    .collect::<Result<Vec<DirEntry>, _>>()
                    .map_err(|_| ())
            })
            .and_then(|entries| {
                entries
                    .iter()
                    .map(|entry| entry.file_name().to_str().map(|x| x.to_string()).ok_or(()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn exists(&self, name: &str) -> bool {
        Path::new(&format!("{}/{name}", self.instance_dir)).exists()
    }

    fn load(&self, name: &str) -> Result<Instance, Error> {
        if !self.exists(name) {
            return Result::Err(Error::UnknownInstance(name.to_string()));
        }

        self.fs
            .open_file(&format!("{}/{name}/machine.yaml", self.instance_dir))
            .and_then(|mut file| Instance::deserialize(name, &mut file))
            .or(Ok(Instance {
                name: name.to_string(),
                user: USER.to_string(),
                cpus: 1,
                mem: util::human_readable_to_bytes("1G").unwrap(),
                disk_capacity: util::human_readable_to_bytes("1G").unwrap(),
                ssh_port: util::generate_random_ssh_port(),
                ..Instance::default()
            }))
    }

    fn store(&self, instance: &Instance) -> Result<(), Error> {
        let path = format!("{}/{}", self.instance_dir, &instance.name);
        let file_name = format!("{path}/machine.yaml");
        let temp_file_name = format!("{file_name}.tmp");

        let mut file = self.fs.create_file(&temp_file_name)?;
        instance.serialize(&mut file)?;
        self.fs.rename_file(&temp_file_name, &file_name)
    }

    fn clone(&self, instance: &Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs.copy_dir(
                &format!("{}/{}", self.instance_dir, instance.name),
                &format!("{}/{new_name}", self.instance_dir),
            )
        }
    }

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs.rename_file(
                &format!("{}/{}", self.instance_dir, instance.name),
                &format!("{}/{new_name}", self.instance_dir),
            )?;
            instance.name = new_name.to_string();
            Result::Ok(())
        }
    }

    fn resize(&self, instance: &mut Instance, size: u64) -> Result<(), Error> {
        if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else if instance.disk_capacity >= size {
            Result::Err(Error::CannotShrinkDisk(instance.name.to_string()))
        } else {
            SystemCommand::new("qemu-img")
                .arg("resize")
                .arg(format!(
                    "{}/{}/machine.img",
                    self.instance_dir, instance.name
                ))
                .arg(size.to_string())
                .run()?;
            instance.disk_capacity = size;
            Result::Ok(())
        }
    }

    fn delete(&self, instance: &Instance) -> Result<(), Error> {
        if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs
                .remove_dir(&format!("{}/{}", self.cache_dir, instance.name))
                .ok();
            self.fs
                .remove_dir(&format!("{}/{}", self.instance_dir, instance.name))
                .ok();
            Ok(())
        }
    }

    fn get_state(&self, instance: &Instance) -> InstanceState {
        if self.is_running(instance) {
            if PortChecker::new(instance.ssh_port).try_connect() {
                InstanceState::Running
            } else {
                InstanceState::Starting
            }
        } else {
            InstanceState::Stopped
        }
    }

    fn is_running(&self, instance: &Instance) -> bool {
        self.fs
            .path_exists(&format!("{}/{}/qemu.pid", self.cache_dir, instance.name))
    }

    fn get_pid(&self, instance: &Instance) -> Result<u64, ()> {
        let pid = self
            .fs
            .read_file_to_string(&format!("{}/{}/qemu.pid", self.cache_dir, instance.name))
            .map_err(|_| ())?;

        pid.trim().parse::<u64>().map_err(|_| ())
    }

    fn get_monitor(&self, instance: &Instance) -> Result<Monitor, Error> {
        Monitor::new(&format!("{}/{}", self.cache_dir, &instance.name))
    }
}
