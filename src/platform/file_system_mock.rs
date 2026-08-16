use crate::error::{Error, FsOperation, Result};
use crate::platform::{FileSystem, SystemMock};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

// Files and directories as two flat path keyed collections. They live in one
// struct because most operations, from a lookup to a rename, have to consult
// both.
#[derive(Default)]
pub struct FileSystemMock {
    files: HashMap<PathBuf, Vec<u8>>,
    dirs: HashSet<PathBuf>,
}

impl FileSystemMock {
    // Registers every ancestor as a directory, so a seeded path behaves like
    // one on a real file system where its parents necessarily exist.
    fn add_parent_dirs(&mut self, path: &Path) {
        let mut parent = path.parent();
        while let Some(dir) = parent {
            if dir.as_os_str().is_empty() {
                break;
            }
            self.dirs.insert(dir.to_path_buf());
            parent = dir.parent();
        }
    }

    fn set_file(&mut self, path: &Path, content: &[u8]) {
        self.files.insert(path.to_path_buf(), content.to_vec());
    }

    // Seeds a file the way a test states one exists, which implies its parents
    // exist too. A write through the `FileSystem` trait uses `set_file`
    // instead, since a real write does not conjure directories.
    fn add_file(&mut self, path: &Path, content: &[u8]) {
        self.add_parent_dirs(path);
        self.set_file(path, content);
    }

    fn add_dir(&mut self, path: &Path) {
        self.add_parent_dirs(path);
        self.dirs.insert(path.to_path_buf());
    }

    fn append_to_file(&mut self, path: &Path, buf: &[u8]) {
        self.files
            .entry(path.to_path_buf())
            .or_default()
            .extend_from_slice(buf);
    }

    fn exists_path(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.dirs.contains(path)
    }

    fn exists_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }

    fn get_file(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.get(path).cloned()
    }

    fn get_path_size(&self, path: &Path) -> u64 {
        if let Some(content) = self.files.get(path) {
            return content.len() as u64;
        }
        self.files
            .iter()
            .filter(|(file, _)| file.starts_with(path))
            .map(|(_, content)| content.len() as u64)
            .sum()
    }

    fn list_children(&self, path: &Path) -> Vec<PathBuf> {
        let mut children: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|file| file.parent() == Some(path))
            .cloned()
            .collect();
        children.extend(
            self.dirs
                .iter()
                .filter(|dir| dir.parent() == Some(path))
                .cloned(),
        );
        children
    }

    fn remove_file(&mut self, path: &Path) -> Option<Vec<u8>> {
        self.files.remove(path)
    }

    fn remove_tree(&mut self, path: &Path) {
        self.dirs.retain(|dir| !dir.starts_with(path));
        self.files.retain(|file, _| !file.starts_with(path));
    }

    // Reports whether anything moved, leaving it to the caller to turn a miss
    // into an error. A directory rename must carry its nested files/dirs along
    // too, since they're tracked as separate flat-path entries here rather
    // than as children of the renamed directory.
    fn rename_path(&mut self, from: &Path, to: &Path) -> bool {
        let file = self.files.remove(from);
        let was_dir = self.dirs.remove(from);

        if file.is_none() && !was_dir {
            return false;
        }

        if let Some(content) = file {
            self.files.insert(to.to_path_buf(), content);
        }
        if was_dir {
            self.dirs.insert(to.to_path_buf());
        }

        let nested_files: Vec<PathBuf> = self
            .files
            .keys()
            .filter(|path| path.starts_with(from))
            .cloned()
            .collect();
        for old_path in nested_files {
            let content = self.files.remove(&old_path).unwrap();
            let new_path = to.join(old_path.strip_prefix(from).unwrap());
            self.files.insert(new_path, content);
        }

        let nested_dirs: Vec<PathBuf> = self
            .dirs
            .iter()
            .filter(|path| path.starts_with(from))
            .cloned()
            .collect();
        for old_path in nested_dirs {
            self.dirs.remove(&old_path);
            let new_path = to.join(old_path.strip_prefix(from).unwrap());
            self.dirs.insert(new_path);
        }

        true
    }
}

// Writes commit into the shared file system state immediately (no buffer, no
// flush-on-drop), so a create_file -> write -> rename_file sequence on the
// same path sees the write's effect without depending on the writer being
// flushed or dropped first, matching how a real File writes through.
struct FileWriterMock {
    path: PathBuf,
    file_system: Rc<RefCell<FileSystemMock>>,
}

impl Write for FileWriterMock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file_system
            .borrow_mut()
            .append_to_file(&self.path, buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl SystemMock {
    pub fn add_file(self, path: &str, content: &[u8]) -> Self {
        self.file_system
            .borrow_mut()
            .add_file(Path::new(path), content);
        self
    }

    pub fn add_dir(self, path: &str) -> Self {
        self.file_system.borrow_mut().add_dir(Path::new(path));
        self
    }

    pub fn get_written_file(&self, path: &str) -> Option<Vec<u8>> {
        self.file_system.borrow().get_file(Path::new(path))
    }
}

impl FileSystem for SystemMock {
    fn exists_path(&self, path: &Path) -> bool {
        self.file_system.borrow().exists_path(path)
    }

    fn exists_dir(&self, path: &Path) -> bool {
        self.file_system.borrow().exists_dir(path)
    }

    fn get_path_size(&self, path: &Path) -> u64 {
        self.file_system.borrow().get_path_size(path)
    }

    fn create_dir(&self, path: &Path) -> Result<()> {
        self.file_system.borrow_mut().add_dir(path);
        Ok(())
    }

    fn create_writable_dir(&self, path: &Path) -> Result<()> {
        self.create_dir(path)
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        self.file_system.borrow_mut().remove_tree(path);
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let file_system = self.file_system.borrow();
        if !file_system.exists_dir(path) {
            return Err(Error::from_fs(
                FsOperation::ReadDir,
                path,
                io::ErrorKind::NotFound.into(),
            ));
        }
        Ok(file_system.list_children(path))
    }

    fn create_file(&self, path: &Path) -> Result<Box<dyn Write>> {
        self.file_system.borrow_mut().set_file(path, &[]);
        Ok(Box::new(FileWriterMock {
            path: path.to_path_buf(),
            file_system: Rc::clone(&self.file_system),
        }))
    }

    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
        self.file_system
            .borrow()
            .get_file(path)
            .map(|content| Box::new(Cursor::new(content)) as Box<dyn Read>)
            .ok_or_else(|| {
                Error::from_fs(FsOperation::OpenFile, path, io::ErrorKind::NotFound.into())
            })
    }

    fn read_file_to_string(&self, path: &Path) -> Result<String> {
        let content = self.file_system.borrow().get_file(path).ok_or_else(|| {
            Error::from_fs(FsOperation::ReadFile, path, io::ErrorKind::NotFound.into())
        })?;
        String::from_utf8(content).map_err(|e| {
            Error::from_fs(
                FsOperation::ReadFile,
                path,
                io::Error::new(io::ErrorKind::InvalidData, e),
            )
        })
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        self.file_system.borrow_mut().set_file(path, contents);
        Ok(())
    }

    fn write_secret_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        self.write_file(path, contents)
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        if self.file_system.borrow_mut().rename_path(from, to) {
            Ok(())
        } else {
            Err(Error::RenameFile {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
                source: io::ErrorKind::NotFound.into(),
            })
        }
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.file_system
            .borrow_mut()
            .remove_file(path)
            .map(|_| ())
            .ok_or_else(|| {
                Error::from_fs(
                    FsOperation::RemoveFile,
                    path,
                    io::ErrorKind::NotFound.into(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::{FileSystem, SystemMock};
    use std::io::{Read, Write};
    use std::path::Path;

    #[test]
    fn exists_path_returns_true_for_seeded_files_and_dirs() {
        let system = SystemMock::new()
            .add_file("/data/foo.txt", b"hello")
            .add_dir("/data/bar");

        assert!(system.exists_path(Path::new("/data/foo.txt")));
        assert!(system.exists_path(Path::new("/data/bar")));
        assert!(!system.exists_path(Path::new("/data/missing")));
    }

    #[test]
    fn exists_path_ignores_a_trailing_slash() {
        let system = SystemMock::new().add_dir("/data/bar");

        assert!(system.exists_path(Path::new("/data/bar/")));
    }

    #[test]
    fn exists_dir_holds_only_for_directories() {
        let system = SystemMock::new()
            .add_file("/data/foo.txt", b"hello")
            .add_dir("/data/bar");

        assert!(system.exists_dir(Path::new("/data/bar")));
        assert!(!system.exists_dir(Path::new("/data/foo.txt")));
        assert!(!system.exists_dir(Path::new("/data/missing")));
    }

    #[test]
    fn add_file_registers_its_parent_dirs() {
        let system = SystemMock::new().add_file("/data/sub/foo.txt", b"hello");

        assert!(system.exists_dir(Path::new("/data/sub")));
        assert!(system.exists_dir(Path::new("/data")));
    }

    #[test]
    fn create_dir_registers_missing_parents() {
        let system = SystemMock::new();

        system.create_dir(Path::new("/data/sub/deep")).unwrap();

        assert!(system.exists_dir(Path::new("/data/sub/deep")));
        assert!(system.exists_dir(Path::new("/data/sub")));
        assert!(system.exists_dir(Path::new("/data")));
    }

    #[test]
    fn get_path_size_returns_file_length() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        assert_eq!(system.get_path_size(Path::new("/data/foo.txt")), 5);
    }

    #[test]
    fn get_path_size_sums_directory_contents_recursively() {
        let system = SystemMock::new()
            .add_file("/data/a.txt", b"12")
            .add_dir("/data/sub")
            .add_file("/data/sub/b.txt", b"345");

        assert_eq!(system.get_path_size(Path::new("/data")), 5);
    }

    #[test]
    fn read_file_to_string_returns_seeded_content() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        assert_eq!(
            system
                .read_file_to_string(Path::new("/data/foo.txt"))
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn read_file_to_string_fails_when_missing() {
        let system = SystemMock::new();

        assert!(
            system
                .read_file_to_string(Path::new("/data/missing"))
                .is_err()
        );
    }

    #[test]
    fn create_file_then_get_written_file_returns_written_bytes() {
        let system = SystemMock::new();

        {
            let mut file = system.create_file(Path::new("/data/out.txt")).unwrap();
            file.write_all(b"hello ").unwrap();
            file.write_all(b"world").unwrap();
        }

        assert_eq!(
            system.get_written_file("/data/out.txt"),
            Some(b"hello world".to_vec())
        );
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
    fn write_file_seeds_readable_content() {
        let system = SystemMock::new();

        system
            .write_file(Path::new("/data/foo.txt"), b"hello")
            .unwrap();

        assert_eq!(
            system
                .read_file_to_string(Path::new("/data/foo.txt"))
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn read_dir_lists_direct_children_only() {
        let system = SystemMock::new()
            .add_file("/data/a.txt", b"1")
            .add_file("/data/b.txt", b"2")
            .add_dir("/data/sub")
            .add_file("/data/sub/c.txt", b"3");

        let mut entries = system
            .read_dir(Path::new("/data"))
            .unwrap()
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        entries.sort();

        assert_eq!(
            entries,
            vec![
                "/data/a.txt".to_string(),
                "/data/b.txt".to_string(),
                "/data/sub".to_string(),
            ]
        );
    }

    #[test]
    fn read_dir_fails_when_the_directory_is_missing() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        assert!(system.read_dir(Path::new("/data/missing")).is_err());
        assert!(system.read_dir(Path::new("/data/foo.txt")).is_err());
    }

    #[test]
    fn rename_file_moves_content_to_new_path() {
        let system = SystemMock::new().add_file("/data/old.txt", b"hello");

        system
            .rename_file(Path::new("/data/old.txt"), Path::new("/data/new.txt"))
            .unwrap();

        assert!(!system.exists_path(Path::new("/data/old.txt")));
        assert_eq!(
            system
                .read_file_to_string(Path::new("/data/new.txt"))
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn rename_file_fails_when_source_missing() {
        let system = SystemMock::new();

        assert!(
            system
                .rename_file(Path::new("/data/old.txt"), Path::new("/data/new.txt"))
                .is_err()
        );
    }

    #[test]
    fn rename_file_moves_a_directory_and_its_contents() {
        let system = SystemMock::new()
            .add_dir("/data/old")
            .add_file("/data/old/a.txt", b"1")
            .add_dir("/data/old/sub")
            .add_file("/data/old/sub/b.txt", b"2");

        system
            .rename_file(Path::new("/data/old"), Path::new("/data/new"))
            .unwrap();

        assert!(!system.exists_path(Path::new("/data/old")));
        assert!(system.exists_path(Path::new("/data/new")));
        assert_eq!(
            system
                .read_file_to_string(Path::new("/data/new/a.txt"))
                .unwrap(),
            "1"
        );
        assert!(system.exists_path(Path::new("/data/new/sub")));
        assert_eq!(
            system
                .read_file_to_string(Path::new("/data/new/sub/b.txt"))
                .unwrap(),
            "2"
        );
    }

    #[test]
    fn create_file_then_rename_file_sees_the_written_content() {
        let system = SystemMock::new();

        let mut file = system.create_file(Path::new("/data/old.tmp")).unwrap();
        file.write_all(b"hello").unwrap();
        // Rename while `file` is still alive (not yet flushed or dropped),
        // mirroring InstanceDao::store and WebClient::download_file's
        // create_file -> write -> rename_file shape.
        system
            .rename_file(Path::new("/data/old.tmp"), Path::new("/data/new"))
            .unwrap();

        assert_eq!(
            system.read_file_to_string(Path::new("/data/new")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn remove_file_deletes_seeded_file() {
        let system = SystemMock::new().add_file("/data/foo.txt", b"hello");

        system.remove_file(Path::new("/data/foo.txt")).unwrap();

        assert!(!system.exists_path(Path::new("/data/foo.txt")));
    }

    #[test]
    fn remove_dir_deletes_directory_and_descendants() {
        let system = SystemMock::new()
            .add_dir("/data/sub")
            .add_file("/data/sub/c.txt", b"3");

        system.remove_dir(Path::new("/data/sub")).unwrap();

        assert!(!system.exists_path(Path::new("/data/sub")));
        assert!(!system.exists_path(Path::new("/data/sub/c.txt")));
    }
}
