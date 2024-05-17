use crate::error::Error;
use std::env;
use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string};

pub fn get_ssh_pub_keys() -> Result<Vec<String>, Error> {
    let home_dir = env::var("HOME").map_err(|_| Error::MissingSshKey)?;

    let mut keys = Vec::new();

    for entry in read_dir(format!("{home_dir}/.ssh"))
        .map_err(|_| Error::MissingSshKey)?
        .flatten()
    {
        let path = entry.path();
        let path = path.as_path();
        if path.extension() == Some(OsStr::new("pub")) {
            if let Ok(key) = read_to_string(path) {
                keys.push(key.trim().to_string());
            }
        }
    }

    if keys.is_empty() {
        return Result::Err(Error::MissingSshKey);
    }

    Ok(keys)
}
