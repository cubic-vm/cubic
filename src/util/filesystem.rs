use crate::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

pub fn copy_file(from: &str, to: &str) -> Result<(), Error> {
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|_| Error::CannotCopyFile(from.to_string(), to.to_string()))
}

pub fn copy_dir(from: &str, to: &str) -> Result<(), Error> {
    Command::new("cp")
        .arg("--recursive")
        .arg(from)
        .arg(to)
        .spawn()
        .map_err(|_| Error::CannotCopyDir(from.to_string(), to.to_string()))?
        .wait()
        .map(|_| ())
        .map_err(|_| Error::CannotCopyDir(from.to_string(), to.to_string()))
}

pub fn move_dir(from: &str, to: &str) -> Result<(), Error> {
    Command::new("mv")
        .arg(from)
        .arg(to)
        .spawn()
        .map_err(|_| Error::CannotMoveDir(from.to_string(), to.to_string()))?
        .wait()
        .map(|_| ())
        .map_err(|_| Error::CannotMoveDir(from.to_string(), to.to_string()))
}

pub fn open_file(path: &str) -> Result<fs::File, Error> {
    fs::File::open(path).map_err(|_| Error::CannotOpenFile(path.to_string()))
}

pub fn create_file(path: &str) -> Result<fs::File, Error> {
    fs::File::create(path).map_err(|_| Error::CannotCreateFile(path.to_string()))
}

pub fn create_dir(path: &str) -> Result<(), Error> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path).map_err(|_| Error::CannotCreateDir(path.to_string()))?;
    }

    Result::Ok(())
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), Error> {
    create_file(path)?
        .write_all(data)
        .map_err(|_| Error::CannotWriteFile(path.to_string()))
}

pub fn remove_file(path: &str) -> Result<(), Error> {
    fs::remove_file(path).map_err(|_| Error::CannotRemoveFile(path.to_string()))
}

pub fn setup_directory_access(path: &str) -> Result<(), Error> {
    create_dir(path)?;

    let permission = fs::metadata(path)
        .map_err(|_| Error::CannotAccessDir(path.to_string()))?
        .permissions();

    if permission.readonly() {
        return Result::Err(Error::CannotWriteDir(path.to_string()));
    }

    Result::Ok(())
}
