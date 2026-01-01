use crate::error::Error;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct FS;

impl FS {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_dir(&self, path: &str) -> Result<(), Error> {
        if !Path::new(path).exists() {
            return fs::create_dir_all(path)
                .map_err(|e| Error::FS(format!("Cannot create directory '{path}' ({e})")));
        }

        Result::Ok(())
    }

    pub fn copy_dir(&self, from: &str, to: &str) -> Result<(), Error> {
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
            .map_err(|e| {
                Error::FS(format!(
                    "Cannot copy directory from '{from}' to '{to}' ({e})"
                ))
            })
    }

    pub fn read_dir(&self, path: &str) -> Result<fs::ReadDir, Error> {
        fs::read_dir(path).map_err(|e| Error::FS(format!("Cannot read directory '{path}' ({e})")))
    }

    pub fn read_dir_file_names(&self, path: &str) -> Result<Vec<String>, Error> {
        self.read_dir(path).map(|dir| {
            dir.flatten()
                .filter_map(|file| file.file_name().to_str().map(|name| name.to_string()))
                .collect()
        })
    }

    pub fn remove_dir(&self, path: &str) -> Result<(), Error> {
        fs::remove_dir_all(path)
            .map_err(|e| Error::FS(format!("Cannot remove directory '{path}' ({e})")))
    }

    pub fn setup_directory_access(&self, path: &str) -> Result<(), Error> {
        self.create_dir(path)?;

        let permission = fs::metadata(path)
            .map_err(|e| Error::FS(format!("Cannot read directory metadata '{path}' ({e})")))?
            .permissions();

        if permission.readonly() {
            return Err(Error::FS(format!("Cannot write directory '{path}'")));
        }

        Result::Ok(())
    }

    pub fn create_file(&self, path: &str) -> Result<fs::File, Error> {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| Error::FS(format!("Cannot create file '{path}' ({e})")))
    }

    pub fn open_file(&self, path: &str) -> Result<fs::File, Error> {
        fs::File::open(path).map_err(|e| Error::FS(format!("Cannot open file '{path}' ({e})")))
    }

    pub fn path_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), Error> {
        self.create_file(path)?
            .write_all(data)
            .map_err(|e| Error::FS(format!("Cannot write file '{path}' ({e})")))
    }

    pub fn read_file_to_string(&self, path: &str) -> Result<String, Error> {
        fs::read_to_string(path).map_err(|e| Error::FS(format!("Cannot read file '{path}' ({e})")))
    }

    pub fn rename_file(&self, from: &str, to: &str) -> Result<(), Error> {
        fs::rename(from, to)
            .map_err(|e| Error::FS(format!("Cannot rename file from '{from}' to '{to}' ({e})")))
    }

    pub fn remove_file(&self, path: &str) -> Result<(), Error> {
        fs::remove_file(path).map_err(|e| Error::FS(format!("Cannot delete file '{path}' ({e})")))
    }
}
