use crate::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn copy_file(from: &str, to: &str) -> Result<(), Error> {
    fs::copy(from, to).map(|_| ()).map_err(Error::Io)
}

pub fn copy_dir(from: &str, to: &str) -> Result<(), Error> {
    Command::new("cp")
        .arg("--recursive")
        .arg(from)
        .arg(to)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Error::Io)?
        .wait()
        .map(|_| ())
        .map_err(Error::Io)
}

pub fn move_dir(from: &str, to: &str) -> Result<(), Error> {
    Command::new("mv")
        .arg(from)
        .arg(to)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Error::Io)?
        .wait()
        .map(|_| ())
        .map_err(Error::Io)
}

pub fn open_file(path: &str) -> Result<fs::File, Error> {
    fs::File::open(path).map_err(Error::Io)
}

pub fn create_file(path: &str) -> Result<fs::File, Error> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(Error::Io)
}

pub fn create_dir(path: &str) -> Result<(), Error> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path).map_err(Error::Io)?;
    }

    Result::Ok(())
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), Error> {
    create_file(path)?.write_all(data).map_err(Error::Io)
}

pub fn rename_file(old: &str, new: &str) -> Result<(), Error> {
    fs::rename(old, new).map_err(Error::Io)
}

pub fn remove_file(path: &str) -> Result<(), Error> {
    fs::remove_file(path).map_err(Error::Io)
}

pub fn setup_directory_access(path: &str) -> Result<(), Error> {
    create_dir(path)?;

    let permission = fs::metadata(path).map_err(Error::Io)?.permissions();

    if permission.readonly() {
        return Result::Err(Error::CannotWriteDir(path.to_string()));
    }

    Result::Ok(())
}
