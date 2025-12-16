#[derive(Default, Clone)]
pub struct Environment {
    data_dir: String,
    cache_dir: String,
    runtime_dir: String,
}

impl Environment {
    pub fn new(data_dir: String, cache_dir: String, runtime_dir: String) -> Self {
        Self {
            data_dir,
            cache_dir,
            runtime_dir,
        }
    }

    pub fn get_cache_dir(&self) -> &str {
        &self.cache_dir
    }

    pub fn get_runtime_dir(&self) -> &str {
        &self.runtime_dir
    }

    pub fn get_instance_dir(&self) -> String {
        format!("{}/machines", self.data_dir)
    }

    pub fn get_image_dir(&self) -> String {
        format!("{}/images", self.cache_dir)
    }

    pub fn get_image_file(&self, image: &str) -> String {
        format!("{}/{image}", self.get_image_dir())
    }

    pub fn get_image_cache_file(&self) -> String {
        format!("{}/images.cache", self.cache_dir)
    }

    pub fn get_instance_dir2(&self, instance: &str) -> String {
        format!("{}/{instance}", &self.get_instance_dir())
    }

    pub fn get_instance_yaml_config_file(&self, instance: &str) -> String {
        format!("{}/machine.yaml", &self.get_instance_dir2(instance))
    }

    pub fn get_instance_toml_config_file(&self, instance: &str) -> String {
        format!("{}/instance.toml", &self.get_instance_dir2(instance))
    }

    pub fn get_instance_image_file(&self, instance: &str) -> String {
        format!("{}/machine.img", &self.get_instance_dir2(instance))
    }

    pub fn get_instance_cache_dir(&self, instance: &str) -> String {
        format!("{}/instances/{instance}", &self.get_cache_dir())
    }

    pub fn get_user_data_image_file(&self, instance: &str) -> String {
        format!("{}/user-data.img", &self.get_instance_cache_dir(instance))
    }

    pub fn get_meta_data_file(&self, instance: &str) -> String {
        format!("{}/meta-data", &self.get_instance_cache_dir(instance))
    }

    pub fn get_user_data_file(&self, instance: &str) -> String {
        format!("{}/user-data", &self.get_instance_cache_dir(instance))
    }

    pub fn get_instance_runtime_dir(&self, instance: &str) -> String {
        format!("{}/instances/{instance}", self.runtime_dir)
    }

    pub fn get_qemu_pid_file(&self, instance: &str) -> String {
        format!("{}/qemu.pid", self.get_instance_runtime_dir(instance))
    }

    pub fn get_console_file(&self, instance: &str) -> String {
        format!("{}/console", self.get_instance_runtime_dir(instance))
    }

    pub fn get_monitor_file(&self, instance: &str) -> String {
        format!("{}/monitor.socket", self.get_instance_runtime_dir(instance))
    }

    pub fn get_guest_agent_file(&self, instance: &str) -> String {
        format!(
            "{}/guest-agent.socket",
            self.get_instance_runtime_dir(instance)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        let env = Environment::new(
            "/data/cubic".to_string(),
            "/cache/cubic".to_string(),
            "/runtime/cubic".to_string(),
        );

        assert_eq!(env.get_cache_dir(), "/cache/cubic");
        assert_eq!(env.get_runtime_dir(), "/runtime/cubic");
        assert_eq!(env.get_instance_dir(), "/data/cubic/machines");
        assert_eq!(env.get_image_dir(), "/cache/cubic/images");
        assert_eq!(
            env.get_image_file("debian_bookworm_amd64"),
            "/cache/cubic/images/debian_bookworm_amd64"
        );
        assert_eq!(env.get_image_cache_file(), "/cache/cubic/images.cache");
        assert_eq!(
            env.get_instance_dir2("mymachine"),
            "/data/cubic/machines/mymachine"
        );
        assert_eq!(
            env.get_instance_yaml_config_file("mymachine"),
            "/data/cubic/machines/mymachine/machine.yaml"
        );
        assert_eq!(
            env.get_instance_toml_config_file("mymachine"),
            "/data/cubic/machines/mymachine/instance.toml"
        );
        assert_eq!(
            env.get_instance_image_file("mymachine"),
            "/data/cubic/machines/mymachine/machine.img"
        );
        assert_eq!(
            env.get_instance_cache_dir("mymachine"),
            "/cache/cubic/instances/mymachine"
        );
        assert_eq!(
            env.get_user_data_image_file("mymachine"),
            "/cache/cubic/instances/mymachine/user-data.img"
        );
        assert_eq!(
            env.get_meta_data_file("mymachine"),
            "/cache/cubic/instances/mymachine/meta-data"
        );
        assert_eq!(
            env.get_user_data_file("mymachine"),
            "/cache/cubic/instances/mymachine/user-data"
        );
        assert_eq!(
            env.get_instance_runtime_dir("mymachine"),
            "/runtime/cubic/instances/mymachine"
        );
        assert_eq!(
            env.get_qemu_pid_file("mymachine"),
            "/runtime/cubic/instances/mymachine/qemu.pid"
        );
        assert_eq!(
            env.get_console_file("mymachine"),
            "/runtime/cubic/instances/mymachine/console"
        );
        assert_eq!(
            env.get_monitor_file("mymachine"),
            "/runtime/cubic/instances/mymachine/monitor.socket"
        );
        assert_eq!(
            env.get_guest_agent_file("mymachine"),
            "/runtime/cubic/instances/mymachine/guest-agent.socket"
        );
    }
}
