mod openssh;
mod port_checker;
mod russh;
mod ssh;

use crate::error::Error;
use crate::fs::FS;
pub use openssh::Openssh;
pub use port_checker::PortChecker;
pub use russh::Russh;
pub use ssh::Ssh;
use std::env;
use std::fs::DirEntry;

fn get_ssh_key_dirs() -> Vec<String> {
    ["SNAP_REAL_HOME", "HOME"]
        .iter()
        .filter_map(|var| env::var(var).ok())
        .map(|dir| format!("{dir}/.ssh"))
        .collect()
}

fn get_ssh_keys() -> Vec<DirEntry> {
    get_ssh_key_dirs()
        .iter()
        .filter_map(|dir| FS::new().read_dir(dir).ok())
        .flatten()
        .filter_map(|item| item.ok())
        .filter(|item| {
            item.file_name()
                .to_str()
                .map(|name| name.starts_with("id_"))
                .unwrap_or_default()
        })
        .collect()
}

pub fn get_ssh_private_key_names() -> Result<Vec<String>, Error> {
    let mut keys = Vec::new();

    for entry in get_ssh_keys() {
        let path = entry.path();
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if extension.is_empty() {
            keys.push(path.as_os_str().to_str().unwrap().to_string());
        }
    }

    Ok(keys)
}

pub fn get_ssh_pub_keys() -> Result<Vec<String>, Error> {
    get_ssh_private_key_names().map(|key| {
        key.iter()
            .filter_map(|path| FS::new().read_file_to_string(&format!("{path}.pub")).ok())
            .map(|content| content.trim().to_string())
            .collect()
    })
}
