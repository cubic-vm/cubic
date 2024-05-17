use crate::error::Error;
use std::env;

const HOME_ENV: &str = "HOME";

pub fn get_home_dir() -> Result<String, Error> {
    env::var(HOME_ENV).map_err(|_| Error::UnsetHomeVar)
}
