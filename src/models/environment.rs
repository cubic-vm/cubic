use crate::models::UserName;
use crate::platform::System;
use std::path::{Path, PathBuf};

#[derive(Default, Clone)]
pub struct Environment {
    username: UserName,
    data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Environment {
    pub fn new(username: UserName, data_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            username,
            data_dir,
            cache_dir,
        }
    }

    pub fn get_username(&self) -> &UserName {
        &self.username
    }

    pub fn get_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn get_instance_dir(&self) -> PathBuf {
        self.data_dir.join("machines")
    }

    pub fn get_image_dir(&self) -> PathBuf {
        self.cache_dir.join("images")
    }

    pub fn get_image_file(&self, image: &str) -> PathBuf {
        self.get_image_dir().join(image)
    }

    /// Image list of an older Cubic, removed by `cubic prune`
    pub fn get_image_cache_file(&self) -> PathBuf {
        self.cache_dir.join("images.cache")
    }

    pub fn get_instance_dir2(&self, instance: &str) -> PathBuf {
        self.get_instance_dir().join(instance)
    }

    pub fn get_instance_toml_config_file(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("instance.toml")
    }

    pub fn get_instance_image_file(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("machine.img")
    }

    pub fn get_cloud_init_file(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("cloud-init.iso")
    }

    pub fn get_qemu_pid_file(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("qemu.pid")
    }

    pub fn get_monitor_socket(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("monitor.sock")
    }

    pub fn get_console_socket(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("console.sock")
    }

    pub fn get_ssh_private_key_file(&self, instance: &str) -> PathBuf {
        self.get_instance_dir2(instance).join("ssh_client_key")
    }

    pub fn get_home_ssh_private_key_paths(&self, system: &dyn System) -> Vec<PathBuf> {
        let mut private_keys = Vec::new();

        let search_dirs: Vec<PathBuf> = ["SNAP_REAL_HOME", "HOME"]
            .iter()
            .filter_map(|var| system.read_env_var(var))
            .map(|dir| PathBuf::from(dir).join(".ssh"))
            .collect();

        for dir in search_dirs {
            if let Ok(file_paths) = system.read_dir(&dir) {
                for file_path in file_paths {
                    if file_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.starts_with("id_"))
                        .unwrap_or_default()
                        && file_path.extension().is_none()
                    {
                        private_keys.push(file_path);
                    }
                }
            }
        }

        private_keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn join_all(base: &str, segments: &[&str]) -> PathBuf {
        segments
            .iter()
            .fold(PathBuf::from(base), |path, segment| path.join(segment))
    }

    #[test]
    fn test_paths() {
        let env = Environment::new(
            UserName::from_str("testuser").unwrap(),
            PathBuf::from("/data/cubic"),
            PathBuf::from("/cache/cubic"),
        );

        assert_eq!(env.get_username().as_str(), "testuser");
        assert_eq!(env.get_cache_dir(), Path::new("/cache/cubic"));
        assert_eq!(
            env.get_ssh_private_key_file("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine", "ssh_client_key"])
        );
        assert_eq!(
            env.get_instance_dir(),
            PathBuf::from("/data/cubic").join("machines")
        );
        assert_eq!(
            env.get_image_dir(),
            PathBuf::from("/cache/cubic").join("images")
        );
        assert_eq!(
            env.get_image_file("debian_bookworm_amd64"),
            join_all("/cache/cubic", &["images", "debian_bookworm_amd64"])
        );
        assert_eq!(
            env.get_image_cache_file(),
            PathBuf::from("/cache/cubic").join("images.cache")
        );
        assert_eq!(
            env.get_instance_dir2("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine"])
        );
        assert_eq!(
            env.get_instance_toml_config_file("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine", "instance.toml"])
        );
        assert_eq!(
            env.get_instance_image_file("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine", "machine.img"])
        );
        assert_eq!(
            env.get_cloud_init_file("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine", "cloud-init.iso"])
        );
        assert_eq!(
            env.get_qemu_pid_file("mymachine"),
            join_all("/data/cubic", &["machines", "mymachine", "qemu.pid"])
        );
    }
}
