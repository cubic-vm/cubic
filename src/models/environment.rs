use crate::fs::FS;
use crate::models::UserName;
use crate::platform::System;
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct Environment {
    username: UserName,
    data_dir: String,
    cache_dir: String,
    runtime_dir: String,
}

impl Environment {
    pub fn new(
        username: UserName,
        data_dir: String,
        cache_dir: String,
        runtime_dir: String,
    ) -> Self {
        Self {
            username,
            data_dir,
            cache_dir,
            runtime_dir,
        }
    }

    pub fn get_username(&self) -> &UserName {
        &self.username
    }

    pub fn get_cache_dir(&self) -> &str {
        &self.cache_dir
    }

    pub fn get_runtime_dir(&self) -> &str {
        &self.runtime_dir
    }

    pub fn get_instance_dir(&self) -> String {
        PathBuf::from(&self.data_dir)
            .join("machines")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_image_dir(&self) -> String {
        PathBuf::from(&self.cache_dir)
            .join("images")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_image_file(&self, image: &str) -> String {
        PathBuf::from(self.get_image_dir())
            .join(image)
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_image_cache_file(&self) -> String {
        PathBuf::from(&self.cache_dir)
            .join("images.cache")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_dir2(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_dir())
            .join(instance)
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_yaml_config_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_dir2(instance))
            .join("machine.yaml")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_toml_config_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_dir2(instance))
            .join("instance.toml")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_image_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_dir2(instance))
            .join("machine.img")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_cache_dir(&self, instance: &str) -> String {
        PathBuf::from(&self.cache_dir)
            .join("instances")
            .join(instance)
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_user_data_image_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_cache_dir(instance))
            .join("user-data.img")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_instance_runtime_dir(&self, instance: &str) -> String {
        PathBuf::from(&self.runtime_dir)
            .join("instances")
            .join(instance)
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_qemu_pid_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_runtime_dir(instance))
            .join("qemu.pid")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_ssh_private_key_file(&self, instance: &str) -> String {
        PathBuf::from(self.get_instance_dir2(instance))
            .join("ssh_client_key")
            .to_string_lossy()
            .into_owned()
    }

    pub fn get_home_ssh_private_key_paths(&self, system: &dyn System, fs: &FS) -> Vec<String> {
        let mut private_keys = Vec::new();

        let search_dirs: Vec<String> = ["SNAP_REAL_HOME", "HOME"]
            .iter()
            .filter_map(|var| system.read_env_var(var))
            .map(|dir| {
                PathBuf::from(dir)
                    .join(".ssh")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        for dir in search_dirs {
            if let Ok(file_paths) = fs.read_dir_file_paths(&dir) {
                for file_path in file_paths {
                    if file_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| name.starts_with("id_"))
                        .unwrap_or_default()
                        && file_path.extension().is_none()
                        && let Some(file_path) = file_path.to_str()
                    {
                        private_keys.push(file_path.to_string());
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
            "/data/cubic".to_string(),
            "/cache/cubic".to_string(),
            "/runtime/cubic".to_string(),
        );

        assert_eq!(env.get_username().as_str(), "testuser");
        assert_eq!(env.get_cache_dir(), "/cache/cubic");
        assert_eq!(
            PathBuf::from(env.get_ssh_private_key_file("mymachine")),
            join_all("/data/cubic", &["machines", "mymachine", "ssh_client_key"])
        );
        assert_eq!(env.get_runtime_dir(), "/runtime/cubic");
        assert_eq!(
            PathBuf::from(env.get_instance_dir()),
            PathBuf::from("/data/cubic").join("machines")
        );
        assert_eq!(
            PathBuf::from(env.get_image_dir()),
            PathBuf::from("/cache/cubic").join("images")
        );
        assert_eq!(
            PathBuf::from(env.get_image_file("debian_bookworm_amd64")),
            join_all("/cache/cubic", &["images", "debian_bookworm_amd64"])
        );
        assert_eq!(
            PathBuf::from(env.get_image_cache_file()),
            PathBuf::from("/cache/cubic").join("images.cache")
        );
        assert_eq!(
            PathBuf::from(env.get_instance_dir2("mymachine")),
            join_all("/data/cubic", &["machines", "mymachine"])
        );
        assert_eq!(
            PathBuf::from(env.get_instance_yaml_config_file("mymachine")),
            join_all("/data/cubic", &["machines", "mymachine", "machine.yaml"])
        );
        assert_eq!(
            PathBuf::from(env.get_instance_toml_config_file("mymachine")),
            join_all("/data/cubic", &["machines", "mymachine", "instance.toml"])
        );
        assert_eq!(
            PathBuf::from(env.get_instance_image_file("mymachine")),
            join_all("/data/cubic", &["machines", "mymachine", "machine.img"])
        );
        assert_eq!(
            PathBuf::from(env.get_instance_cache_dir("mymachine")),
            join_all("/cache/cubic", &["instances", "mymachine"])
        );
        assert_eq!(
            PathBuf::from(env.get_user_data_image_file("mymachine")),
            join_all("/cache/cubic", &["instances", "mymachine", "user-data.img"])
        );
        assert_eq!(
            PathBuf::from(env.get_instance_runtime_dir("mymachine")),
            join_all("/runtime/cubic", &["instances", "mymachine"])
        );
        assert_eq!(
            PathBuf::from(env.get_qemu_pid_file("mymachine")),
            join_all("/runtime/cubic", &["instances", "mymachine", "qemu.pid"])
        );
    }
}
