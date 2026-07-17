use crate::error::{Error, Result};
use crate::models::Environment;
use crate::platform::System;

const ROOT_USERNAME: &str = "root";
pub const DEFAULT_USERNAME: &str = "cubic";
const USERNAME_ENV_VARS: [&str; 3] = ["USER", "LOGNAME", "USERNAME"];

pub struct EnvironmentFactory;

impl EnvironmentFactory {
    fn read_env(system: &dyn System, var: &str) -> Result<String> {
        system
            .read_env_var(var)
            .ok_or_else(|| Error::UnsetEnvVar(var.to_string()))
    }

    fn read_current_username(system: &dyn System) -> Option<String> {
        USERNAME_ENV_VARS
            .iter()
            .find_map(|var| system.read_env_var(var))
    }

    pub fn get_username(system: &dyn System) -> String {
        let username =
            Self::read_current_username(system).unwrap_or_else(|| DEFAULT_USERNAME.to_string());
        if username == ROOT_USERNAME {
            DEFAULT_USERNAME.to_string()
        } else {
            username
        }
    }

    #[cfg(target_os = "linux")]
    pub fn create_env(system: &dyn System) -> Result<Environment> {
        let data_dir = Self::read_env(system, "SNAP_USER_COMMON")
            .or_else(|_| Self::read_env(system, "XDG_DATA_HOME"))
            .or_else(|_| {
                Self::read_env(system, "HOME").map(|home| format!("{home}/.local/share"))
            })?;
        let cache_dir = Self::read_env(system, "XDG_CACHE_HOME")
            .or_else(|_| Self::read_env(system, "HOME").map(|home| format!("{home}/.cache")))?;
        let runtime_dir = Self::read_env(system, "XDG_RUNTIME_DIR")
            .or_else(|_| Self::read_env(system, "UID").map(|uid| format!("/run/user/{uid}")))?;

        Ok(Environment::new(
            Self::get_username(system),
            format!("{data_dir}/cubic"),
            format!("{cache_dir}/cubic"),
            format!("{runtime_dir}/cubic"),
        ))
    }

    #[cfg(target_os = "macos")]
    pub fn create_env(system: &dyn System) -> Result<Environment> {
        let home_dir = Self::read_env(system, "HOME")?;

        Ok(Environment::new(
            Self::get_username(system),
            format!("{home_dir}/Library/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
        ))
    }

    #[cfg(target_os = "windows")]
    pub fn create_env(system: &dyn System) -> Result<Environment> {
        let local_app_data_dir = Self::read_env(system, "LOCALAPPDATA")?;
        let temp_dir = Self::read_env(system, "TEMP")?;

        Ok(Environment::new(
            Self::get_username(system),
            format!("{local_app_data_dir}\\cubic"),
            format!("{temp_dir}\\cubic"),
            format!("{temp_dir}\\cubic"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_read_current_username_prefers_user_over_logname_and_username() {
        let system = SystemMock::new()
            .add_env_var("USER", "alice")
            .add_env_var("LOGNAME", "bob")
            .add_env_var("USERNAME", "carol");

        assert_eq!(
            EnvironmentFactory::read_current_username(&system),
            Some("alice".to_string())
        );
    }

    #[test]
    fn test_read_current_username_falls_back_to_logname_then_username() {
        let system = SystemMock::new().add_env_var("USERNAME", "carol");

        assert_eq!(
            EnvironmentFactory::read_current_username(&system),
            Some("carol".to_string())
        );
    }

    #[test]
    fn test_get_username_falls_back_to_default_when_unset() {
        let system = SystemMock::new();

        assert_eq!(EnvironmentFactory::get_username(&system), DEFAULT_USERNAME);
    }

    #[test]
    fn test_get_username_maps_root_to_default() {
        let system = SystemMock::new().add_env_var("USER", ROOT_USERNAME);

        assert_eq!(EnvironmentFactory::get_username(&system), DEFAULT_USERNAME);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_create_env_falls_back_through_xdg_and_home() {
        let system = SystemMock::new()
            .add_env_var("USER", "alice")
            .add_env_var("HOME", "/home/alice")
            .add_env_var("XDG_RUNTIME_DIR", "/run/user/1000");

        let env = EnvironmentFactory::create_env(&system).unwrap();

        assert_eq!(
            env.get_instance_dir(),
            "/home/alice/.local/share/cubic/machines"
        );
        assert_eq!(env.get_cache_dir(), "/home/alice/.cache/cubic");
        assert_eq!(env.get_runtime_dir(), "/run/user/1000/cubic");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_create_env_prefers_snap_and_xdg_vars_over_home() {
        let system = SystemMock::new()
            .add_env_var("USER", "alice")
            .add_env_var("SNAP_USER_COMMON", "/snap/cubic/common")
            .add_env_var("XDG_CACHE_HOME", "/cache")
            .add_env_var("XDG_RUNTIME_DIR", "/run/user/1000")
            .add_env_var("HOME", "/home/alice");

        let env = EnvironmentFactory::create_env(&system).unwrap();

        assert_eq!(env.get_instance_dir(), "/snap/cubic/common/cubic/machines");
        assert_eq!(env.get_cache_dir(), "/cache/cubic");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_create_env_errors_when_no_dir_vars_are_set() {
        let system = SystemMock::new().add_env_var("USER", "alice");

        assert!(EnvironmentFactory::create_env(&system).is_err());
    }
}
