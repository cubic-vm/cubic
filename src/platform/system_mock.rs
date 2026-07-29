#[cfg(test)]
pub mod tests {
    use crate::error::{Error, Result};
    use crate::platform::{Stream, System};
    use std::cell::{Cell, RefCell};
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::io::{Cursor, Read, Write};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    pub struct SystemMock {
        env_vars: HashMap<String, String>,
        output: RefCell<String>,
        terminal: Cell<bool>,
        input: RefCell<VecDeque<String>>,
        files: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
        dirs: RefCell<HashSet<PathBuf>>,
    }

    impl SystemMock {
        pub fn new() -> Self {
            Self {
                env_vars: HashMap::new(),
                output: RefCell::new(String::new()),
                terminal: Cell::new(false),
                input: RefCell::new(VecDeque::new()),
                files: Rc::new(RefCell::new(HashMap::new())),
                dirs: RefCell::new(HashSet::new()),
            }
        }

        pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
            self.env_vars.insert(key.to_string(), value.to_string());
            self
        }

        pub fn set_terminal(self, terminal: bool) -> Self {
            self.terminal.set(terminal);
            self
        }

        pub fn get_output(&self) -> String {
            self.output.borrow().clone()
        }

        pub fn push_input(&self, line: &str) {
            self.input.borrow_mut().push_back(line.to_string());
        }

        pub fn add_file(self, path: &str, content: &[u8]) -> Self {
            self.files
                .borrow_mut()
                .insert(PathBuf::from(path), content.to_vec());
            self
        }

        pub fn add_dir(self, path: &str) -> Self {
            self.dirs.borrow_mut().insert(PathBuf::from(path));
            self
        }

        pub fn get_written_file(&self, path: &str) -> Option<Vec<u8>> {
            self.files.borrow().get(Path::new(path)).cloned()
        }

        fn log(&self, msg: &str) {
            self.output.borrow_mut().push_str(&format!("{msg}\n"));
        }
    }

    // Writes commit into the shared `files` map immediately (no buffer, no
    // flush-on-drop), so a create_file -> write -> rename_file sequence on the
    // same path sees the write's effect without depending on the writer being
    // flushed or dropped first, matching how a real File writes through.
    struct MockFileWriter {
        path: PathBuf,
        files: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
    }

    impl Write for MockFileWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.files
                .borrow_mut()
                .entry(self.path.clone())
                .or_default()
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl System for SystemMock {
        fn read_env_var(&self, key: &str) -> Option<String> {
            self.env_vars.get(key).cloned()
        }

        fn print(&self, _stream: Stream, msg: &str) {
            self.output.borrow_mut().push_str(msg);
        }

        fn println(&self, _stream: Stream, msg: &str) {
            self.log(msg);
        }

        fn flush(&self, _stream: Stream) {}

        fn is_terminal(&self, _stream: Stream) -> bool {
            self.terminal.get()
        }

        fn read_input(&self) -> String {
            self.input
                .borrow_mut()
                .pop_front()
                .unwrap_or_default()
                .trim()
                .to_string()
        }

        fn read_secret(&self) -> std::result::Result<String, ()> {
            Ok(self.input.borrow_mut().pop_front().unwrap_or_default())
        }

        fn raw_mode(&self) {}

        fn reset(&self) {}

        fn exists_path(&self, path: &Path) -> bool {
            self.files.borrow().contains_key(path) || self.dirs.borrow().contains(path)
        }

        fn exists_dir(&self, path: &Path) -> bool {
            self.dirs.borrow().contains(path)
        }

        fn get_path_size(&self, path: &Path) -> u64 {
            let files = self.files.borrow();
            if let Some(content) = files.get(path) {
                return content.len() as u64;
            }
            files
                .iter()
                .filter(|(file, _)| file.starts_with(path))
                .map(|(_, content)| content.len() as u64)
                .sum()
        }

        fn create_dir(&self, path: &Path) -> Result<()> {
            self.dirs.borrow_mut().insert(path.to_path_buf());
            Ok(())
        }

        fn create_writable_dir(&self, path: &Path) -> Result<()> {
            self.create_dir(path)
        }

        fn remove_dir(&self, path: &Path) -> Result<()> {
            self.dirs.borrow_mut().retain(|dir| !dir.starts_with(path));
            self.files
                .borrow_mut()
                .retain(|file, _| !file.starts_with(path));
            Ok(())
        }

        fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
            let mut children: Vec<PathBuf> = self
                .files
                .borrow()
                .keys()
                .filter(|file| file.parent() == Some(path))
                .cloned()
                .collect();
            children.extend(
                self.dirs
                    .borrow()
                    .iter()
                    .filter(|dir| dir.parent() == Some(path))
                    .cloned(),
            );
            Ok(children)
        }

        fn create_file(&self, path: &Path) -> Result<Box<dyn Write>> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), Vec::new());
            Ok(Box::new(MockFileWriter {
                path: path.to_path_buf(),
                files: Rc::clone(&self.files),
            }))
        }

        fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
            self.files
                .borrow()
                .get(path)
                .cloned()
                .map(|content| Box::new(Cursor::new(content)) as Box<dyn Read>)
                .ok_or_else(|| {
                    Error::FS(format!("Cannot open file '{}' (not found)", path.display()))
                })
        }

        fn read_file_to_string(&self, path: &Path) -> Result<String> {
            let content = self.files.borrow().get(path).cloned().ok_or_else(|| {
                Error::FS(format!("Cannot read file '{}' (not found)", path.display()))
            })?;
            String::from_utf8(content)
                .map_err(|e| Error::FS(format!("Cannot read file '{}' ({e})", path.display())))
        }

        fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), contents.to_vec());
            Ok(())
        }

        fn write_secret_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
            self.write_file(path, contents)
        }

        fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
            let file = self.files.borrow_mut().remove(from);
            let was_dir = self.dirs.borrow_mut().remove(from);

            if file.is_none() && !was_dir {
                return Err(Error::FS(format!(
                    "Cannot rename file from '{}' to '{}' (not found)",
                    from.display(),
                    to.display()
                )));
            }

            if let Some(content) = file {
                self.files.borrow_mut().insert(to.to_path_buf(), content);
            }
            if was_dir {
                self.dirs.borrow_mut().insert(to.to_path_buf());
            }

            // A directory rename must carry its nested files/dirs along too,
            // since they're tracked as separate flat-path entries here rather
            // than as children of the renamed directory.
            let nested_files: Vec<PathBuf> = self
                .files
                .borrow()
                .keys()
                .filter(|path| path.starts_with(from))
                .cloned()
                .collect();
            for old_path in nested_files {
                let content = self.files.borrow_mut().remove(&old_path).unwrap();
                let new_path = to.join(old_path.strip_prefix(from).unwrap());
                self.files.borrow_mut().insert(new_path, content);
            }

            let nested_dirs: Vec<PathBuf> = self
                .dirs
                .borrow()
                .iter()
                .filter(|path| path.starts_with(from))
                .cloned()
                .collect();
            for old_path in nested_dirs {
                self.dirs.borrow_mut().remove(&old_path);
                let new_path = to.join(old_path.strip_prefix(from).unwrap());
                self.dirs.borrow_mut().insert(new_path);
            }

            Ok(())
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.files
                .borrow_mut()
                .remove(path)
                .map(|_| ())
                .ok_or_else(|| {
                    Error::FS(format!(
                        "Cannot delete file '{}' (not found)",
                        path.display()
                    ))
                })
        }
    }

    #[test]
    fn read_env_var_returns_configured_value() {
        let system = SystemMock::new().add_env_var("FOO", "bar");

        assert_eq!(system.read_env_var("FOO"), Some("bar".to_string()));
    }

    #[test]
    fn read_env_var_returns_none_when_not_set() {
        let system = SystemMock::new();

        assert_eq!(system.read_env_var("FOO"), None);
    }

    #[test]
    fn println_appends_message_and_newline_to_output() {
        let system = SystemMock::new();

        system.println(Stream::Stdout, "hello");
        system.println(Stream::Stderr, "world");

        assert_eq!(system.get_output(), "hello\nworld\n");
    }

    #[test]
    fn read_input_returns_queued_input() {
        let system = SystemMock::new();
        system.push_input("first");
        system.push_input("second");

        assert_eq!(system.read_input(), "first");
        assert_eq!(system.read_input(), "second");
        assert_eq!(system.read_input(), "");
    }

    #[test]
    fn is_terminal_defaults_to_false() {
        let system = SystemMock::new();

        assert!(!system.is_terminal(Stream::Stdout));
    }
}
