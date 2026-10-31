use crate::error::{Error, FsOperation, Result};
use crate::platform::{Access, FileSystem, SystemMock};
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

// Flat path map where a directory has no content.
#[derive(Default)]
pub struct FileSystemMock {
    paths: HashMap<PathBuf, Option<Vec<u8>>>,
}

impl FileSystemMock {
    fn add_dir(&mut self, path: &Path) {
        for dir in path.ancestors().filter(|dir| !dir.as_os_str().is_empty()) {
            self.paths.entry(dir.to_path_buf()).or_insert(None);
        }
    }

    fn add_file(&mut self, path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            self.add_dir(parent);
        }
        self.set_file(path, content);
    }

    fn set_file(&mut self, path: &Path, content: &[u8]) {
        self.paths
            .insert(path.to_path_buf(), Some(content.to_vec()));
    }

    fn get_file(&self, path: &Path) -> Option<Vec<u8>> {
        self.paths.get(path).cloned().flatten()
    }
}

struct FileWriterMock {
    path: PathBuf,
    file_system: Arc<Mutex<FileSystemMock>>,
}

impl Write for FileWriterMock {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file_system
            .lock()
            .unwrap()
            .paths
            .entry(self.path.clone())
            .or_default()
            .get_or_insert_default()
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SystemMock {
    pub fn add_file(self, path: impl AsRef<Path>, content: &[u8]) -> Self {
        self.lock_file_system().add_file(path.as_ref(), content);
        self
    }

    pub fn add_dir(self, path: impl AsRef<Path>) -> Self {
        self.lock_file_system().add_dir(path.as_ref());
        self
    }

    pub fn get_written_file(&self, path: impl AsRef<Path>) -> Option<Vec<u8>> {
        self.lock_file_system().get_file(path.as_ref())
    }

    fn lock_file_system(&self) -> MutexGuard<'_, FileSystemMock> {
        self.file_system.lock().unwrap()
    }
}

impl FileSystem for SystemMock {
    fn exists_path(&self, path: &Path) -> bool {
        self.lock_file_system().paths.contains_key(path)
    }

    fn get_path_size(&self, path: &Path) -> u64 {
        self.lock_file_system()
            .paths
            .iter()
            .filter(|(entry, _)| entry.starts_with(path))
            .filter_map(|(_, content)| content.as_ref())
            .map(|content| content.len() as u64)
            .sum()
    }

    fn get_available_space(&self, _path: &Path) -> Option<u64> {
        None
    }

    fn create_dir(&self, path: &Path) -> Result<()> {
        self.lock_file_system().add_dir(path);
        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        self.lock_file_system()
            .paths
            .retain(|entry, _| !entry.starts_with(path));
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let file_system = self.lock_file_system();
        if !matches!(file_system.paths.get(path), Some(None)) {
            let error = io::ErrorKind::NotFound.into();
            return Err(Error::from_fs(FsOperation::ReadDir, path, error));
        }
        Ok(file_system
            .paths
            .keys()
            .filter(|entry| entry.parent() == Some(path))
            .cloned()
            .collect())
    }

    fn create_file(&self, path: &Path, _access: Access) -> Result<Box<dyn Write>> {
        self.lock_file_system().set_file(path, &[]);
        Ok(Box::new(FileWriterMock {
            path: path.to_path_buf(),
            file_system: Arc::clone(&self.file_system),
        }))
    }

    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
        self.lock_file_system()
            .get_file(path)
            .map(|content| Box::new(Cursor::new(content)) as Box<dyn Read>)
            .ok_or_else(|| {
                Error::from_fs(FsOperation::OpenFile, path, io::ErrorKind::NotFound.into())
            })
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        let mut file_system = self.lock_file_system();
        if !file_system.paths.contains_key(from) {
            return Err(Error::RenameFile {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
                source: io::ErrorKind::NotFound.into(),
            });
        }

        let moved: Vec<PathBuf> = file_system
            .paths
            .keys()
            .filter(|entry| entry.starts_with(from))
            .cloned()
            .collect();
        for old in moved {
            let content = file_system.paths.remove(&old).unwrap();
            let rest = old.strip_prefix(from).unwrap();
            let new = to.components().chain(rest.components()).collect();
            file_system.paths.insert(new, content);
        }
        Ok(())
    }

    fn copy_file(&self, from: &Path, to: &Path) -> Result<()> {
        let mut file_system = self.lock_file_system();
        let content = file_system.get_file(from).ok_or_else(|| Error::CopyFile {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            source: io::ErrorKind::NotFound.into(),
        })?;
        file_system.set_file(to, &content);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        let mut file_system = self.lock_file_system();
        if !matches!(file_system.paths.get(path), Some(Some(_))) {
            let error = io::ErrorKind::NotFound.into();
            return Err(Error::from_fs(FsOperation::RemoveFile, path, error));
        }
        file_system.paths.remove(path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::{Access, FileSystem, SystemMock};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};

    #[test]
    fn exists_path_finds_files_and_dirs() {
        let system = SystemMock::new()
            .add_file("/data/foo.txt", b"hello")
            .add_dir("/data/bar");

        assert!(system.exists_path(Path::new("/data/foo.txt")));
        assert!(system.exists_path(Path::new("/data/bar/")));
        assert!(!system.exists_path(Path::new("/data/missing")));
    }

    #[test]
    fn add_file_and_create_dir_register_parent_dirs() {
        let system = SystemMock::new().add_file("/data/sub/foo.txt", b"hello");
        system.create_dir(Path::new("/other/sub/deep")).unwrap();

        for dir in [
            "/data/sub",
            "/data",
            "/other/sub/deep",
            "/other/sub",
            "/other",
        ] {
            assert!(system.read_dir(Path::new(dir)).is_ok(), "{dir}");
        }
    }

    #[test]
    fn get_path_size_sums_files_recursively() {
        let system = SystemMock::new()
            .add_file("/data/a.txt", b"12")
            .add_file("/data/sub/b.txt", b"345");

        assert_eq!(system.get_path_size(Path::new("/data/a.txt")), 2);
        assert_eq!(system.get_path_size(Path::new("/data")), 5);
    }

    #[test]
    fn create_file_then_get_written_file_returns_written_bytes() {
        let system = SystemMock::new();

        let mut file = system
            .create_file(Path::new("/data/out.txt"), Access::Normal)
            .unwrap();
        file.write_all(b"hello ").unwrap();
        file.write_all(b"world").unwrap();

        let written = system.get_written_file("/data/out.txt");
        assert_eq!(written, Some(b"hello world".to_vec()));
    }

    #[test]
    fn open_file_reads_back_seeded_content() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        let mut buf = String::new();
        system
            .open_file(Path::new("/data/foo.txt"))
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        assert_eq!(buf, "hello");
    }

    #[test]
    fn read_dir_lists_direct_children_only() {
        let system = SystemMock::new()
            .add_file("/data/a.txt", b"1")
            .add_dir("/data/sub")
            .add_file("/data/sub/c.txt", b"3");

        let mut entries = system.read_dir(Path::new("/data")).unwrap();
        entries.sort();

        assert_eq!(
            entries,
            [PathBuf::from("/data/a.txt"), PathBuf::from("/data/sub")]
        );
    }

    #[test]
    fn read_dir_fails_when_the_directory_is_missing() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        assert!(system.read_dir(Path::new("/data/missing")).is_err());
        assert!(system.read_dir(Path::new("/data/foo.txt")).is_err());
    }

    #[test]
    fn rename_file_moves_a_directory_and_its_contents() {
        let system = SystemMock::new()
            .add_file("/data/old/a.txt", b"1")
            .add_file("/data/old/sub/b.txt", b"2");

        system
            .rename_file(Path::new("/data/old"), Path::new("/data/new"))
            .unwrap();

        assert!(!system.exists_path(Path::new("/data/old")));
        assert!(system.read_dir(Path::new("/data/new/sub")).is_ok());
        let written = system.get_written_file("/data/new/a.txt");
        assert_eq!(written, Some(b"1".to_vec()));
        let written = system.get_written_file("/data/new/sub/b.txt");
        assert_eq!(written, Some(b"2".to_vec()));
    }

    #[test]
    fn copy_file_keeps_the_source_and_writes_the_target() {
        let system = SystemMock::new().add_file("/data/old.txt", b"hello");

        system
            .copy_file(Path::new("/data/old.txt"), Path::new("/data/new.txt"))
            .unwrap();

        assert!(system.exists_path(Path::new("/data/old.txt")));
        let written = system.get_written_file("/data/new.txt");
        assert_eq!(written, Some(b"hello".to_vec()));
    }

    #[test]
    fn rename_copy_and_remove_fail_when_source_missing() {
        let system = SystemMock::new();
        let (from, to) = (Path::new("/data/old.txt"), Path::new("/data/new.txt"));

        assert!(system.rename_file(from, to).is_err());
        assert!(system.copy_file(from, to).is_err());
        assert!(system.remove_file(from).is_err());
    }

    #[test]
    fn remove_file_and_remove_dir_delete_their_targets() {
        let system = SystemMock::new()
            .add_file("/data/foo.txt", b"hello")
            .add_file("/data/sub/c.txt", b"3");

        system.remove_file(Path::new("/data/foo.txt")).unwrap();
        system.remove_dir(Path::new("/data/sub")).unwrap();

        assert!(!system.exists_path(Path::new("/data/foo.txt")));
        assert!(!system.exists_path(Path::new("/data/sub")));
        assert!(!system.exists_path(Path::new("/data/sub/c.txt")));
    }
}
