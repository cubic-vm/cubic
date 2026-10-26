use crate::error::{Error, FsOperation, Result};
use crate::platform::{Access, FileSystem, OsSystem};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use sysinfo::{DiskRefreshKind, Disks};

impl FileSystem for OsSystem {
    fn exists_path(&self, path: &Path) -> bool {
        path.exists()
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

    fn get_available_space(&self, path: &Path) -> Option<u64> {
        let path = path.canonicalize().ok()?;

        Disks::new_with_refreshed_list_specifics(DiskRefreshKind::nothing().with_storage())
            .list()
            .iter()
            .filter(|disk| path.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().as_os_str().len())
            .map(|disk| disk.available_space())
    }

    // Creates a directory only the owner may enter. Windows inherits the ACL.
    fn create_dir(&self, path: &Path) -> Result<()> {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }

        builder
            .create(path)
            .map_err(|e| Error::from_fs(FsOperation::CreateDir, path, e))
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).map_err(|e| Error::from_fs(FsOperation::RemoveDir, path, e))
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        fs::read_dir(path)
            .map(|dir| dir.flatten().map(|entry| entry.path()).collect())
            .map_err(|e| Error::from_fs(FsOperation::ReadDir, path, e))
    }

    fn create_file(&self, path: &Path, access: Access) -> Result<Box<dyn Write>> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);

        #[cfg(unix)]
        if access == Access::Private {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(not(unix))]
        let _ = access;

        options
            .open(path)
            .map(|f| Box::new(f) as Box<dyn Write>)
            .map_err(|e| Error::from_fs(FsOperation::CreateFile, path, e))
    }

    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
        fs::File::open(path)
            .map(|f| Box::new(f) as Box<dyn Read>)
            .map_err(|e| Error::from_fs(FsOperation::OpenFile, path, e))
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to).map_err(|e| Error::RenameFile {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            source: e,
        })
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::copy(from, to).map(|_| ()).map_err(|e| Error::CopyFile {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            source: e,
        })
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path).map_err(|e| Error::from_fs(FsOperation::RemoveFile, path, e))
    }
}
