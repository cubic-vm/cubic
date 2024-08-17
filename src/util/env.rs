use crate::error::Error;
use std::env;

const HOME_ENV: &str = "HOME";
const XDG_RUNTIME_DIR_ENV: &str = "XDG_RUNTIME_DIR";

pub fn get_home_dir() -> Result<String, Error> {
    env::var(HOME_ENV).map_err(|_| Error::UnsetEnvVar(HOME_ENV.to_string()))
}

pub fn get_xdg_runtime_dir() -> Result<String, Error> {
    env::var(XDG_RUNTIME_DIR_ENV).map_err(|_| Error::UnsetEnvVar(XDG_RUNTIME_DIR_ENV.to_string()))
}
