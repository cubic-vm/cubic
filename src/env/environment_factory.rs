use crate::env::Environment;
use crate::error::{Error, Result};
use std::env;

const ROOT_USERNAME: &str = "root";
pub const DEFAULT_USERNAME: &str = "cubic";

pub struct EnvironmentFactory;

impl EnvironmentFactory {
    fn read_env(var: &str) -> Result<String> {
        env::var(var).map_err(|_| Error::UnsetEnvVar(var.to_string()))
    }

    pub fn get_username() -> String {
        let username = whoami::username().unwrap_or(DEFAULT_USERNAME.to_string());
        if username == ROOT_USERNAME {
            DEFAULT_USERNAME.to_string()
        } else {
            username
        }
    }

    #[cfg(target_os = "linux")]
    pub fn create_env() -> Result<Environment> {
        let data_dir = Self::read_env("SNAP_USER_COMMON")
            .or(Self::read_env("XDG_DATA_HOME"))
            .or_else(|_| Self::read_env("HOME").map(|home| format!("{home}/.local/share")))?;
        let cache_dir = Self::read_env("XDG_CACHE_HOME")
            .or_else(|_| Self::read_env("HOME").map(|home| format!("{home}/.cache")))?;
        let runtime_dir = Self::read_env("XDG_RUNTIME_DIR")
            .or_else(|_| Self::read_env("UID").map(|uid| format!("/run/user/{uid}")))?;

        Ok(Environment::new(
            Self::get_username(),
            format!("{data_dir}/cubic"),
            format!("{cache_dir}/cubic"),
            format!("{runtime_dir}/cubic"),
        ))
    }

    #[cfg(target_os = "macos")]
    pub fn create_env() -> Result<Environment> {
        let home_dir = Self::read_env("HOME")?;

        Ok(Environment::new(
            Self::get_username(),
            format!("{home_dir}/Library/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
            format!("{home_dir}/Library/Caches/cubic"),
        ))
    }

    #[cfg(target_os = "windows")]
    pub fn create_env() -> Result<Environment> {
        let local_app_data_dir = Self::read_env("LOCALAPPDATA")?;
        let temp_dir = Self::read_env("TEMP")?;

        Ok(Environment::new(
            Self::get_username(),
            format!("{local_app_data_dir}/cubic"),
            format!("{temp_dir}/cubic"),
            format!("{temp_dir}/cubic"),
        ))
    }
}
