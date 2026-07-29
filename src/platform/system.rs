use crate::error::Result;
use crate::platform::Stream;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub trait System {
    fn read_env_var(&self, key: &str) -> Option<String>;

    fn print(&self, stream: Stream, msg: &str);
    fn println(&self, stream: Stream, msg: &str);
    fn flush(&self, stream: Stream);
    fn is_terminal(&self, stream: Stream) -> bool;

    fn read_input(&self) -> String;
    fn read_secret(&self) -> std::result::Result<String, ()>;

    fn raw_mode(&self);
    fn reset(&self);

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
