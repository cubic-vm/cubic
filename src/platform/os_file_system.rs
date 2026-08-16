use crate::error::{Error, FsOperation, Result};
use crate::platform::{FileSystem, OsSystem};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

impl FileSystem for OsSystem {
    fn exists_path(&self, path: &Path) -> bool {
        path.exists()
    }

    fn exists_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn get_path_size(&self, path: &Path) -> u64 {
        fs::metadata(path)
            .map(|metadata| {
                if metadata.is_dir() {
                    fs::read_dir(path)
                        .map(|dir| {
                            dir.flatten()
                                .map(|entry| self.get_path_size(&entry.path()))
                                .sum()
                        })
                        .unwrap_or_default()
                } else {
                    metadata.len()
                }
            })
            .unwrap_or_default()
    }

    fn create_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(|e| Error::from_fs(FsOperation::CreateDir, path, e))
    }

    fn create_writable_dir(&self, path: &Path) -> Result<()> {
        self.create_dir(path)?;

        let permission = fs::metadata(path)
            .map_err(|e| Error::from_fs(FsOperation::ReadMetadata, path, e))?
            .permissions();

        if permission.readonly() {
            return Err(Error::ReadOnlyDir(path.to_path_buf()));
        }

        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).map_err(|e| Error::from_fs(FsOperation::RemoveDir, path, e))
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        fs::read_dir(path)
            .map(|dir| dir.flatten().map(|entry| entry.path()).collect())
            .map_err(|e| Error::from_fs(FsOperation::ReadDir, path, e))
    }

    fn create_file(&self, path: &Path) -> Result<Box<dyn Write>> {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map(|f| Box::new(f) as Box<dyn Write>)
            .map_err(|e| Error::from_fs(FsOperation::CreateFile, path, e))
    }

    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
        fs::File::open(path)
            .map(|f| Box::new(f) as Box<dyn Read>)
            .map_err(|e| Error::from_fs(FsOperation::OpenFile, path, e))
    }

    fn read_file_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| Error::from_fs(FsOperation::ReadFile, path, e))
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        fs::write(path, contents).map_err(|e| Error::from_fs(FsOperation::WriteFile, path, e))
    }

    // Writes a file only the owner may read, for private keys and other
    // secrets. On Windows the file inherits the directory ACL instead.
    fn write_secret_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        options
            .open(path)
            .and_then(|mut file| file.write_all(contents))
            .map_err(|e| Error::from_fs(FsOperation::WriteFile, path, e))
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to).map_err(|e| Error::RenameFile {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            source: e,
        })
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path).map_err(|e| Error::from_fs(FsOperation::RemoveFile, path, e))
    }
}
