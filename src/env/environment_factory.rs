use crate::env::Environment;
use crate::error::Error;
use std::env;

pub struct EnvironmentFactory;

impl EnvironmentFactory {
    fn read_env(var: &str) -> Result<String, Error> {
        env::var(var).map_err(|_| Error::UnsetEnvVar(var.to_string()))
    }

    #[cfg(target_os = "linux")]
    pub fn create_env() -> Result<Environment, Error> {
        let data_dir = Self::read_env("SNAP_USER_COMMON")
            .or(Self::read_env("XDG_DATA_HOME"))
            .or_else(|_| Self::read_env("HOME").map(|home| format!("{home}/.local/share")))?;
        let cache_dir = Self::read_env("XDG_CACHE_HOME")
            .or_else(|_| Self::read_env("HOME").map(|home| format!("{home}/.cache")))?;
        let runtime_dir = Self::read_env("XDG_RUNTIME_DIR")
            .or_else(|_| Self::read_env("UID").map(|uid| format!("/run/user/{uid}")))?;

        Ok(Environment::new(
            format!("{data_dir}/cubic"),
            format!("{cache_dir}/cubic"),
            format!("{runtime_dir}/cubic"),
        ))
    }

    #[cfg(target_os = "macos")]
    pub fn create_env() -> Result<Environment, Error> {
        let home_dir = Self::read_env("HOME")?;

        Ok(Environment::new(
            format!("{home_dir}/Library/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
        ))
    }

    #[cfg(target_os = "windows")]
    pub fn create_env() -> Result<Environment, Error> {
        let local_app_data_dir = Self::read_env("LOCALAPPDATA")?;
        let temp_dir = Self::read_env("TEMP")?;

        Ok(Environment::new(
            format!("{local_app_data_dir}/cubic"),
            format!("{temp_dir}/cubic"),
            format!("{temp_dir}/cubic"),
        ))
    }
}
