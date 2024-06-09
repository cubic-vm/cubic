use crate::error::Error;
use crate::util;
use std::env;
use std::ffi::OsStr;
use std::fs::{read_dir, read_to_string};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

pub struct SSHClient {
    pub port: u16,
}

impl SSHClient {
    pub fn new(port: u16) -> Self {
        SSHClient { port }
    }

    pub fn try_connect(&self) -> bool {
        let mut buf = [0];
        TcpStream::connect(format!("127.0.0.1:{}", &self.port))
            .and_then(|stream| {
                stream.set_read_timeout(Some(Duration::new(0, 100000000)))?;
                stream.peek(&mut buf)
            })
            .is_ok()
    }
}

pub fn check_ssh_key() {
    let home_dir = env::var("HOME").unwrap();
    util::create_dir(&format!("{home_dir}/.ssh")).ok();

    if util::get_ssh_key_names().unwrap_or_default().is_empty() {
        println!("No SSH keys found. Please generate a SSH key:");
        Command::new("ssh-keygen")
            .arg("-t")
            .arg("ed25519")
            .arg("-f")
            .arg(format!("{home_dir}/.ssh/id_ed25519"))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Could not run ssh-keygen #1")
            .wait()
            .expect("Could not run ssh-keygen #2");
    }
}

pub fn get_ssh_key_names() -> Result<Vec<String>, Error> {
    let home_dir = env::var("HOME").map_err(|_| Error::MissingSshKey)?;

    let mut keys = Vec::new();

    if let Ok(entries) = read_dir(format!("{home_dir}/.ssh")) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap_or_default();
            let path = entry.path();
            let extension = path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if file_name.starts_with("id_") && extension.is_empty() {
                keys.push(path.as_os_str().to_str().unwrap().to_string());
            }
        }
    }

    Ok(keys)
}

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
