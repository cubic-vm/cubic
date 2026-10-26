use crate::error::{Error, FsOperation, Result};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub trait FileSystem {
    fn exists_path(&self, path: &Path) -> bool;
    fn get_path_size(&self, path: &Path) -> u64;
    fn get_available_space(&self, path: &Path) -> Option<u64>;
    fn create_dir(&self, path: &Path) -> Result<()>;
    fn create_writable_dir(&self, path: &Path) -> Result<()>;
    fn remove_dir(&self, path: &Path) -> Result<()>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn create_file(&self, path: &Path) -> Result<Box<dyn Write>>;
    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn write_secret_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn rename_file(&self, from: &Path, to: &Path) -> Result<()>;
    fn copy_file(&self, from: &Path, to: &Path) -> Result<()>;
    fn remove_file(&self, path: &Path) -> Result<()>;

    fn read_file_to_string(&self, path: &Path) -> Result<String> {
        let mut content = String::new();
        self.open_file(path)?
            .read_to_string(&mut content)
            .map_err(|e| Error::from_fs(FsOperation::ReadFile, path, e))?;
        Ok(content)
    }
}
