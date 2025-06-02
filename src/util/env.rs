use crate::error::Error;
use std::env;

const HOME_ENV: &str = "HOME";
const SNAP_COMMON_ENV: &str = "SNAP_USER_COMMON";
const XDG_RUNTIME_DIR_ENV: &str = "XDG_RUNTIME_DIR";

pub fn get_data_dir() -> Result<String, Error> {
    env::var(SNAP_COMMON_ENV).or(env::var(HOME_ENV)
        .map(|home_dir| format!("{home_dir}/.local/share"))
        .map_err(|_| Error::UnsetEnvVar(HOME_ENV.to_string())))
}

pub fn get_instance_data_dir() -> Result<String, Error> {
    get_data_dir().map(|dir| format!("{dir}/cubic/machines"))
}

pub fn get_image_data_dir() -> Result<String, Error> {
    get_data_dir().map(|dir| format!("{dir}/cubic/images"))
}

pub fn get_xdg_runtime_dir() -> Result<String, Error> {
    env::var(XDG_RUNTIME_DIR_ENV).map_err(|_| Error::UnsetEnvVar(XDG_RUNTIME_DIR_ENV.to_string()))
}

pub fn get_image_cache_file() -> Result<String, Error> {
    get_xdg_runtime_dir().map(|dir| format!("{dir}/cubic/images.cache"))
}
