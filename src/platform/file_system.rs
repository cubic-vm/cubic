use crate::error::Result;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub trait FileSystem {
    fn exists_path(&self, path: &Path) -> bool;
    fn exists_dir(&self, path: &Path) -> bool;
    fn get_path_size(&self, path: &Path) -> u64;
    fn create_dir(&self, path: &Path) -> Result<()>;
    fn create_writable_dir(&self, path: &Path) -> Result<()>;
    fn remove_dir(&self, path: &Path) -> Result<()>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn create_file(&self, path: &Path) -> Result<Box<dyn Write>>;
    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>>;
    fn read_file_to_string(&self, path: &Path) -> Result<String>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn write_secret_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn rename_file(&self, from: &Path, to: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;
}
