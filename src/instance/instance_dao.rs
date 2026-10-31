use crate::error::{Error, Result};
use crate::instance::{InstanceSerializer, InstanceStore, TomlInstanceDeserializer};
use crate::models::{DataSize, Environment, Instance, InstanceName, MIN_DISK};
use crate::platform::{Access, System};
use crate::qemu::QemuImg;
use std::str;
use std::str::FromStr;
use std::sync::Arc;

pub struct InstanceDao {
    pub env: Environment,
    system: Arc<dyn System>,
}

impl InstanceDao {
    pub fn new(system: Arc<dyn System>, env: &Environment) -> Result<Self> {
        system.create_dir(&env.get_instance_dir())?;
        system.create_dir(env.get_cache_dir())?;

        Ok(InstanceDao {
            env: env.clone(),
            system,
        })
    }
}

impl InstanceStore for InstanceDao {
    fn get_instances(&self) -> Vec<String> {
        let mut instances: Vec<String> = self
            .system
            .read_dir(&self.env.get_instance_dir())
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
        self.system.exists_path(&self.env.get_instance_dir2(name))
    }

    fn load(&self, name: &str) -> Result<Instance> {
        if !self.exists(name) {
            return Err(Error::UnknownInstance(name.to_string()));
        }

        let path = self.env.get_instance_toml_config_file(name);

        let mut file = self
            .system
            .open_file(&path)
            .map_err(|error| Error::from_config(name, error))?;

        let mut instance = TomlInstanceDeserializer::new().deserialize(name, &mut file)?;
        QemuImg::new(self.system.as_ref()).read_disk_info(&self.env, &mut instance);

        Ok(instance)
    }

    fn store(&self, instance: &Instance) -> Result<()> {
        let file_name = self.env.get_instance_toml_config_file(&instance.name);
        let temp_file_name = file_name.with_added_extension("tmp");

        let content = InstanceSerializer::new().serialize(instance)?;
        self.system
            .write_file(&temp_file_name, content.as_bytes(), Access::Normal)?;
        self.system.rename_file(&temp_file_name, &file_name)?;

        Ok(())
    }

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<()> {
        let old_dir = self.env.get_instance_dir2(&instance.name);
        let new_dir = self.env.get_instance_dir2(new_name);
        self.system.rename_file(&old_dir, &new_dir)?;
        instance.name = new_name.to_string();
        Ok(())
    }

    fn resize(&self, instance: &mut Instance, size: u64) -> Result<()> {
        if (size as usize) < MIN_DISK {
            Err(Error::DiskBelowMinimum(
                instance.name.to_string(),
                DataSize::new(MIN_DISK).to_size(),
            ))
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
        self.system
            .remove_dir(&self.env.get_instance_dir2(&instance.name))
            .ok();
        Ok(())
    }

    fn create_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
        if instance.has_snapshot(name) {
            Err(Error::SnapshotAlreadyExists(
                instance.name.to_string(),
                name.to_string(),
            ))
        } else {
            QemuImg::new(self.system.as_ref())
                .create_snapshot(&self.env.get_instance_image_file(&instance.name), name)
        }
    }

    fn restore_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
        if !instance.has_snapshot(name) {
            Err(Error::UnknownSnapshot(
                instance.name.to_string(),
                name.to_string(),
            ))
        } else {
            QemuImg::new(self.system.as_ref())
                .restore_snapshot(&self.env.get_instance_image_file(&instance.name), name)
        }
    }

    fn delete_snapshot(&self, instance: &Instance, name: &str) -> Result<()> {
        if !instance.has_snapshot(name) {
            Err(Error::UnknownSnapshot(
                instance.name.to_string(),
                name.to_string(),
            ))
        } else {
            QemuImg::new(self.system.as_ref())
                .delete_snapshot(&self.env.get_instance_image_file(&instance.name), name)
        }
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
            "/data".into(),
            "/cache".into(),
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
        let dao = InstanceDao::new(Arc::new(system), &build_env()).unwrap();

        assert!(dao.exists("test"));
        assert!(!dao.exists("missing"));
    }

    #[test]
    fn test_store_then_load_round_trips_instance() {
        let system = SystemMock::new().add_dir("/data/machines/test");
        let dao = InstanceDao::new(Arc::new(system), &build_env()).unwrap();
        let instance = Instance {
            name: "test".to_string(),
            ..Instance::default()
        };

        dao.store(&instance).unwrap();
        let loaded = dao.load("test").unwrap();

        assert_eq!(loaded.name, instance.name);
    }

    #[test]
    fn test_load_reports_a_missing_config() {
        let system = SystemMock::new().add_dir("/data/machines/test");
        let dao = InstanceDao::new(Arc::new(system), &build_env()).unwrap();

        assert!(matches!(
            dao.load("test"),
            Err(Error::InvalidInstanceConfig { name, .. }) if name == "test"
        ));
    }

    fn build_instance_with_snapshot() -> Instance {
        Instance {
            name: "test".to_string(),
            snapshots: vec![crate::models::Snapshot {
                name: "clean".to_string(),
            }],
            ..Instance::default()
        }
    }

    #[test]
    fn test_create_snapshot_runs_qemu_img() {
        let env = build_env();
        let command = format!(
            "qemu-img snapshot -c clean {}",
            env.get_instance_image_file("test").display()
        );
        let system = Arc::new(SystemMock::new().add_command_output(&command, b""));
        let dao = InstanceDao::new(system.clone(), &env).unwrap();

        dao.create_snapshot(&build_instance(), "clean").unwrap();

        assert_eq!(system.get_executed_commands(), vec![command]);
    }

    #[test]
    fn test_create_snapshot_rejects_a_duplicate_name() {
        let dao = InstanceDao::new(Arc::new(SystemMock::new()), &build_env()).unwrap();

        assert!(matches!(
            dao.create_snapshot(&build_instance_with_snapshot(), "clean"),
            Err(Error::SnapshotAlreadyExists(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
    }

    #[test]
    fn test_restore_snapshot_rejects_an_unknown_name() {
        let dao = InstanceDao::new(Arc::new(SystemMock::new()), &build_env()).unwrap();

        assert!(matches!(
            dao.restore_snapshot(&build_instance(), "clean"),
            Err(Error::UnknownSnapshot(instance, snapshot))
                if instance == "test" && snapshot == "clean"
        ));
    }

    #[test]
    fn test_get_instances_lists_sorted_valid_names() {
        let system = SystemMock::new()
            .add_dir("/data/machines/zebra")
            .add_dir("/data/machines/apple");
        let dao = InstanceDao::new(Arc::new(system), &build_env()).unwrap();

        assert_eq!(dao.get_instances(), vec!["apple", "zebra"]);
    }
}
