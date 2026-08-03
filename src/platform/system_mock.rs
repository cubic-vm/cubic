#[cfg(test)]
pub mod tests {
    use crate::error::{Error, Result};
    use crate::platform::{ReadWrite, Stream, System};
    use crate::util::SystemCommand;
    use std::cell::RefCell;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::io::{Cursor, Read, Write};
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::time::Duration;

    // How a seeded pid answers a kill. Every state is visible to a liveness
    // check, they differ only in what killing one does.
    #[derive(Clone, Copy)]
    enum ProcessState {
        // Dies when killed.
        Alive,
        // Stays alive and reports a failure, standing in for a kill the host
        // rejects, such as one aimed at another user's process.
        Unkillable,
        // Already gone once the kill lands, modelling the race between
        // checking a pid and signalling it.
        Vanished,
    }

    // What the host reports about its own size.
    struct HostResources {
        total_memory: u64,
        available_memory: u64,
        cpu_count: u16,
    }

    // Both halves of the console, so a test can seed input and read back
    // output through a single piece of state.
    #[derive(Default)]
    struct ConsoleState {
        output: String,
        input: VecDeque<String>,
    }

    impl ConsoleState {
        fn print(&mut self, msg: &str) {
            self.output.push_str(msg);
        }

        fn println(&mut self, msg: &str) {
            self.output.push_str(&format!("{msg}\n"));
        }

        fn push_input(&mut self, line: &str) {
            self.input.push_back(line.to_string());
        }

        // An empty line once the queue is drained, standing in for a console
        // that has nothing left to read.
        fn pop_input(&mut self) -> String {
            self.input.pop_front().unwrap_or_default()
        }
    }

    // Files and directories as two flat path keyed collections. They live in
    // one struct because most operations, from a lookup to a rename, have to
    // consult both.
    #[derive(Default)]
    struct FileSystemState {
        files: HashMap<PathBuf, Vec<u8>>,
        dirs: HashSet<PathBuf>,
    }

    impl FileSystemState {
        // Registers every ancestor as a directory, so a seeded path behaves
        // like one on a real file system where its parents necessarily exist.
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

        // Seeds a file the way a test states one exists, which implies its
        // parents exist too. A write through the `System` trait uses
        // `set_file` instead, since a real write does not conjure directories.
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

        // Reports whether anything moved, leaving it to the caller to turn a
        // miss into an error. A directory rename must carry its nested
        // files/dirs along too, since they're tracked as separate flat-path
        // entries here rather than as children of the renamed directory.
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

    // The pids the host knows about, alongside the record of which ones a kill
    // actually took down.
    #[derive(Default)]
    struct ProcessTable {
        processes: HashMap<u64, ProcessState>,
        killed: Vec<u64>,
    }

    impl ProcessTable {
        fn add(&mut self, pid: u64, state: ProcessState) {
            self.processes.insert(pid, state);
        }

        fn exists(&self, pid: u64) -> bool {
            self.processes.contains_key(&pid)
        }

        fn kill(&mut self, pid: u64) -> Result<()> {
            match self.processes.get(&pid).copied() {
                None => Err(Error::ProcessNotFound(pid)),
                Some(ProcessState::Unkillable) => Err(Error::KillFailed(pid)),
                Some(ProcessState::Vanished) => {
                    self.processes.remove(&pid);
                    Err(Error::ProcessNotFound(pid))
                }
                Some(ProcessState::Alive) => {
                    self.processes.remove(&pid);
                    self.killed.push(pid);
                    Ok(())
                }
            }
        }
    }

    // Where a bind starts looking. A bind skips every port the host already
    // knows about, so seeding one is always safe, but a test that names a port
    // without seeding it is not protected. The base sits high to keep that out
    // of the range tests reach for by hand.
    const FIRST_FREE_PORT: u16 = 60000;

    // What a listening port sends the moment a connection lands. Only the
    // presence of these bytes carries meaning, never their content.
    const GREETING: &[u8] = b"SSH-2.0-mock\r\n";

    // What a port does when it is dialled. A port the host has never heard of
    // is one nothing listens on and nothing has claimed, so it is free.
    #[derive(Clone, Copy)]
    enum PortState {
        // Accepts and greets the caller, the way a ready service does.
        Listening,
        // Accepts and then says nothing, standing in for a forward that is
        // bound before the service behind it can answer.
        Silent,
        // Handed out by a bind. Nothing listens on it, since a bind only
        // reserves the number and leaves the listening to whoever asked.
        Bound,
    }

    // Every port the host knows about, in the order it learned of them,
    // alongside the record of which ones were dialled. A port that is absent
    // is one nothing listens on, so connecting to it is refused rather than
    // quietly succeeding.
    #[derive(Default)]
    struct NetworkState {
        ports: Vec<(u16, PortState)>,
        connected: Vec<u16>,
    }

    impl NetworkState {
        fn add(&mut self, port: u16, state: PortState) {
            self.ports.retain(|(known, _)| *known != port);
            self.ports.push((port, state));
        }

        fn connect(&mut self, port: u16) -> Result<Box<dyn ReadWrite>> {
            self.connected.push(port);

            match self.find(port) {
                Some(PortState::Listening) => Ok(Box::new(MockStream::new(GREETING))),
                Some(PortState::Silent) => Ok(Box::new(MockStream::new(b""))),
                _ => Err(Error::Io(std::io::ErrorKind::ConnectionRefused.into())),
            }
        }

        // Takes the lowest port nothing has claimed yet and records the claim,
        // so a second bind cannot hand out the same number.
        fn bind(&mut self) -> Result<u16> {
            let port = (FIRST_FREE_PORT..=u16::MAX)
                .find(|port| self.find(*port).is_none())
                .ok_or(Error::NoPortAvailable)?;
            self.add(port, PortState::Bound);
            Ok(port)
        }

        fn find(&self, port: u16) -> Option<PortState> {
            self.ports
                .iter()
                .find(|(known, _)| *known == port)
                .map(|(_, state)| *state)
        }
    }

    // A connection to a seeded listener. Reading past the greeting reports a
    // timeout rather than end of file, because a real socket held open by a
    // peer that has stopped talking blocks until its read timeout expires. A
    // reader that treats end of file as success would see a silent port as a
    // talking one.
    struct MockStream {
        greeting: Cursor<Vec<u8>>,
    }

    impl MockStream {
        fn new(greeting: &[u8]) -> Self {
            Self {
                greeting: Cursor::new(greeting.to_vec()),
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            match self.greeting.read(buf)? {
                0 if !buf.is_empty() => Err(std::io::ErrorKind::TimedOut.into()),
                count => Ok(count),
            }
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // How a seeded command answers a run.
    enum CommandResult {
        // Exits successfully with this on stdout.
        Output(Vec<u8>),
        // Exits with a failure carrying this on stderr.
        Failure(String),
    }

    // The commands the host knows how to answer, alongside the record of what
    // was attempted. A command that is not seeded is one the host cannot run,
    // so it reports a missing binary rather than quietly succeeding.
    #[derive(Default)]
    struct CommandTable {
        results: HashMap<String, CommandResult>,
        executed: Vec<String>,
    }

    impl CommandTable {
        fn add(&mut self, command: &str, result: CommandResult) {
            self.results.insert(command.to_string(), result);
        }

        fn run(&mut self, command: &SystemCommand) -> Result<Vec<u8>> {
            let line = command.get_command();
            self.executed.push(line.clone());

            match self.results.get(line.as_str()) {
                None => Err(Error::SystemCommandNotFound(
                    command.get_program().to_string(),
                )),
                Some(CommandResult::Failure(stderr)) => {
                    Err(Error::SystemCommandFailed(line, stderr.clone()))
                }
                Some(CommandResult::Output(stdout)) => Ok(stdout.clone()),
            }
        }
    }

    pub struct SystemMock {
        env_vars: HashMap<String, String>,
        terminal: bool,
        host: HostResources,
        console: RefCell<ConsoleState>,
        file_system: Rc<RefCell<FileSystemState>>,
        processes: RefCell<ProcessTable>,
        commands: RefCell<CommandTable>,
        network: RefCell<NetworkState>,
    }

    impl SystemMock {
        pub fn new() -> Self {
            Self {
                env_vars: HashMap::new(),
                terminal: false,
                // A roomy host by default, so a test that does not care about
                // host size never hits a resource limit by accident.
                host: HostResources {
                    total_memory: 16 * 1024 * 1024 * 1024,
                    available_memory: 16 * 1024 * 1024 * 1024,
                    cpu_count: 8,
                },
                console: RefCell::new(ConsoleState::default()),
                file_system: Rc::new(RefCell::new(FileSystemState::default())),
                processes: RefCell::new(ProcessTable::default()),
                commands: RefCell::new(CommandTable::default()),
                network: RefCell::new(NetworkState::default()),
            }
        }

        pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
            self.env_vars.insert(key.to_string(), value.to_string());
            self
        }

        pub fn set_terminal(mut self, terminal: bool) -> Self {
            self.terminal = terminal;
            self
        }

        pub fn get_output(&self) -> String {
            self.console.borrow().output.clone()
        }

        pub fn push_input(&self, line: &str) {
            self.console.borrow_mut().push_input(line);
        }

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

        pub fn add_process(self, pid: u64) -> Self {
            self.add_process_state(pid, ProcessState::Alive)
        }

        pub fn add_unkillable_process(self, pid: u64) -> Self {
            self.add_process_state(pid, ProcessState::Unkillable)
        }

        pub fn add_vanishing_process(self, pid: u64) -> Self {
            self.add_process_state(pid, ProcessState::Vanished)
        }

        fn add_process_state(self, pid: u64, state: ProcessState) -> Self {
            self.processes.borrow_mut().add(pid, state);
            self
        }

        pub fn set_host_resources(
            mut self,
            total_memory: u64,
            available_memory: u64,
            cpu_count: u16,
        ) -> Self {
            self.host = HostResources {
                total_memory,
                available_memory,
                cpu_count,
            };
            self
        }

        pub fn get_killed_processes(&self) -> Vec<u64> {
            self.processes.borrow().killed.clone()
        }

        pub fn add_command_output(self, command: &str, stdout: &[u8]) -> Self {
            self.commands
                .borrow_mut()
                .add(command, CommandResult::Output(stdout.to_vec()));
            self
        }

        pub fn add_failing_command(self, command: &str, stderr: &str) -> Self {
            self.commands
                .borrow_mut()
                .add(command, CommandResult::Failure(stderr.to_string()));
            self
        }

        // Every command the host was asked to run, seeded or not, in order.
        pub fn get_executed_commands(&self) -> Vec<String> {
            self.commands.borrow().executed.clone()
        }

        // A port that accepts and then greets the caller, the way a ready sshd
        // does.
        pub fn add_open_port(self, port: u16) -> Self {
            self.add_port_state(port, PortState::Listening)
        }

        // A port that accepts and then says nothing, the way QEMU hostfwd does
        // while the guest is still booting.
        pub fn add_silent_port(self, port: u16) -> Self {
            self.add_port_state(port, PortState::Silent)
        }

        fn add_port_state(self, port: u16, state: PortState) -> Self {
            self.network.borrow_mut().add(port, state);
            self
        }

        // Every port the host was asked to connect to, seeded or not, in order.
        pub fn get_connected_ports(&self) -> Vec<u16> {
            self.network.borrow().connected.clone()
        }
    }

    // Writes commit into the shared file system state immediately (no buffer,
    // no flush-on-drop), so a create_file -> write -> rename_file sequence on
    // the same path sees the write's effect without depending on the writer
    // being flushed or dropped first, matching how a real File writes through.
    struct MockFileWriter {
        path: PathBuf,
        file_system: Rc<RefCell<FileSystemState>>,
    }

    impl Write for MockFileWriter {
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

    impl System for SystemMock {
        fn read_env_var(&self, key: &str) -> Option<String> {
            self.env_vars.get(key).cloned()
        }

        fn print(&self, _stream: Stream, msg: &str) {
            self.console.borrow_mut().print(msg);
        }

        fn println(&self, _stream: Stream, msg: &str) {
            self.console.borrow_mut().println(msg);
        }

        fn flush(&self, _stream: Stream) {}

        fn is_terminal(&self, _stream: Stream) -> bool {
            self.terminal
        }

        fn read_input(&self) -> String {
            self.console.borrow_mut().pop_input().trim().to_string()
        }

        fn read_secret(&self) -> std::result::Result<String, ()> {
            Ok(self.console.borrow_mut().pop_input())
        }

        fn raw_mode(&self) {}

        fn reset(&self) {}

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
                return Err(Error::FS(format!(
                    "Cannot read directory '{}' (not found)",
                    path.display()
                )));
            }
            Ok(file_system.list_children(path))
        }

        fn create_file(&self, path: &Path) -> Result<Box<dyn Write>> {
            self.file_system.borrow_mut().set_file(path, &[]);
            Ok(Box::new(MockFileWriter {
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
                    Error::FS(format!("Cannot open file '{}' (not found)", path.display()))
                })
        }

        fn read_file_to_string(&self, path: &Path) -> Result<String> {
            let content = self.file_system.borrow().get_file(path).ok_or_else(|| {
                Error::FS(format!("Cannot read file '{}' (not found)", path.display()))
            })?;
            String::from_utf8(content)
                .map_err(|e| Error::FS(format!("Cannot read file '{}' ({e})", path.display())))
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
                Err(Error::FS(format!(
                    "Cannot rename file from '{}' to '{}' (not found)",
                    from.display(),
                    to.display()
                )))
            }
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.file_system
                .borrow_mut()
                .remove_file(path)
                .map(|_| ())
                .ok_or_else(|| {
                    Error::FS(format!(
                        "Cannot delete file '{}' (not found)",
                        path.display()
                    ))
                })
        }

        // The timeout has no meaning here, since a seeded listener answers at
        // once and an unseeded one refuses at once.
        fn connect_port(&self, port: u16, _timeout: Duration) -> Result<Box<dyn ReadWrite>> {
            self.network.borrow_mut().connect(port)
        }

        fn bind_port(&self) -> Result<u16> {
            self.network.borrow_mut().bind()
        }

        fn exists_process(&self, pid: u64) -> bool {
            self.processes.borrow().exists(pid)
        }

        fn kill_process(&self, pid: u64) -> Result<()> {
            self.processes.borrow_mut().kill(pid)
        }

        fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>> {
            self.commands.borrow_mut().run(command)
        }

        // A detached start has nothing to wait for, so it only reports whether
        // the host could launch the command at all.
        fn spawn_command(&self, command: &SystemCommand) -> Result<()> {
            self.commands.borrow_mut().run(command).map(|_| ())
        }

        fn get_total_memory(&self) -> u64 {
            self.host.total_memory
        }

        fn get_available_memory(&self) -> u64 {
            self.host.available_memory
        }

        fn get_cpu_count(&self) -> u16 {
            self.host.cpu_count
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

    #[test]
    fn exists_process_only_finds_seeded_pids() {
        let system = SystemMock::new().add_process(42);

        assert!(system.exists_process(42));
        assert!(!system.exists_process(43));
    }

    #[test]
    fn kill_process_records_the_pid_and_ends_the_process() {
        let system = SystemMock::new().add_process(42);

        system.kill_process(42).unwrap();

        assert_eq!(system.get_killed_processes(), vec![42]);
        assert!(!system.exists_process(42));
    }

    #[test]
    fn kill_process_fails_for_an_unkillable_pid_that_stays_alive() {
        let system = SystemMock::new().add_unkillable_process(42);

        assert!(matches!(
            system.kill_process(42),
            Err(Error::KillFailed(42))
        ));
        assert!(system.exists_process(42));
        assert!(system.get_killed_processes().is_empty());
    }

    #[test]
    fn kill_process_reports_a_vanishing_process_as_gone() {
        let system = SystemMock::new().add_vanishing_process(42);

        assert!(system.exists_process(42));
        assert!(matches!(
            system.kill_process(42),
            Err(Error::ProcessNotFound(42))
        ));
        // Once the kill has reported it gone, every later look agrees.
        assert!(!system.exists_process(42));
        assert!(system.get_killed_processes().is_empty());
    }

    #[test]
    fn seeding_a_pid_twice_keeps_the_last_state() {
        let system = SystemMock::new().add_process(42).add_vanishing_process(42);

        assert!(matches!(
            system.kill_process(42),
            Err(Error::ProcessNotFound(42))
        ));

        let system = SystemMock::new().add_vanishing_process(7).add_process(7);

        system.kill_process(7).unwrap();
        assert_eq!(system.get_killed_processes(), vec![7]);
    }

    #[test]
    fn kill_process_fails_for_an_unknown_pid() {
        let system = SystemMock::new();

        assert!(matches!(
            system.kill_process(42),
            Err(Error::ProcessNotFound(42))
        ));
        assert!(system.get_killed_processes().is_empty());
    }

    #[test]
    fn host_resources_return_the_configured_values() {
        let system = SystemMock::new().set_host_resources(8_000, 2_000, 4);

        assert_eq!(system.get_total_memory(), 8_000);
        assert_eq!(system.get_available_memory(), 2_000);
        assert_eq!(system.get_cpu_count(), 4);
    }

    #[test]
    fn run_command_returns_the_seeded_output() {
        let system = SystemMock::new().add_command_output("echo hello", b"hello\n");

        let mut command = SystemCommand::new("echo");
        command.arg("hello");

        assert_eq!(system.run_command(&command).unwrap(), b"hello\n");
    }

    #[test]
    fn run_command_fails_for_an_unseeded_command() {
        let system = SystemMock::new();

        assert!(matches!(
            system.run_command(&SystemCommand::new("qemu-img")),
            Err(Error::SystemCommandNotFound(program)) if program == "qemu-img"
        ));
    }

    #[test]
    fn run_command_reports_a_seeded_failure_with_its_stderr() {
        let system = SystemMock::new().add_failing_command("qemu-img resize", "no such file");

        let mut command = SystemCommand::new("qemu-img");
        command.arg("resize");

        assert!(matches!(
            system.run_command(&command),
            Err(Error::SystemCommandFailed(line, stderr))
                if line == "qemu-img resize" && stderr == "no such file"
        ));
    }

    #[test]
    fn run_command_matches_on_the_arguments() {
        let system = SystemMock::new().add_command_output("qemu-img info a", b"a");

        let mut other = SystemCommand::new("qemu-img");
        other.arg("info").arg("b");

        assert!(system.run_command(&other).is_err());
    }

    #[test]
    fn spawn_command_succeeds_without_returning_output() {
        let system = SystemMock::new().add_command_output("qemu-system-x86_64", b"ignored");

        system
            .spawn_command(&SystemCommand::new("qemu-system-x86_64"))
            .unwrap();
    }

    #[test]
    fn spawn_command_fails_for_an_unseeded_command() {
        let system = SystemMock::new();

        assert!(matches!(
            system.spawn_command(&SystemCommand::new("qemu-system-x86_64")),
            Err(Error::SystemCommandNotFound(program)) if program == "qemu-system-x86_64"
        ));
    }

    #[test]
    fn get_executed_commands_records_every_attempt_in_order() {
        let system = SystemMock::new().add_command_output("echo one", b"");

        let mut first = SystemCommand::new("echo");
        first.arg("one");
        system.run_command(&first).unwrap();

        let mut second = SystemCommand::new("echo");
        second.arg("two");
        system.run_command(&second).ok();

        // The failed attempt is recorded too, since the host was still asked
        // to run it.
        assert_eq!(system.get_executed_commands(), vec!["echo one", "echo two"]);
    }

    #[test]
    fn connect_port_reads_something_back_from_an_open_port() {
        let system = SystemMock::new().add_open_port(22);

        let mut stream = system.connect_port(22, Duration::from_secs(1)).unwrap();

        // Only the presence of the bytes matters, since that is what tells a
        // service that can answer from one that cannot yet.
        assert!(stream.read(&mut [0]).unwrap() > 0);
    }

    #[test]
    fn connect_port_times_out_reading_a_silent_port() {
        let system = SystemMock::new().add_silent_port(22);

        let mut stream = system.connect_port(22, Duration::from_secs(1)).unwrap();

        // A peer that has sent nothing has to look like a stalled read rather
        // than an ended one, otherwise a caller that treats end of file as
        // success reads a silent port as a talking one.
        assert_eq!(
            stream.read(&mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::TimedOut
        );
    }

    #[test]
    fn connect_port_is_refused_when_nothing_listens() {
        let system = SystemMock::new();

        assert!(matches!(
            system.connect_port(22, Duration::from_secs(1)),
            Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused
        ));
    }

    #[test]
    fn get_connected_ports_records_every_attempt_in_order() {
        let system = SystemMock::new().add_open_port(22);

        system.connect_port(22, Duration::from_secs(1)).unwrap();
        system.connect_port(80, Duration::from_secs(1)).ok();

        assert_eq!(system.get_connected_ports(), vec![22, 80]);
    }

    #[test]
    fn bind_port_claims_successive_free_ports() {
        let system = SystemMock::new();

        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT);
        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT + 1);
    }

    #[test]
    fn bind_port_skips_a_port_the_test_already_claimed() {
        let system = SystemMock::new().add_open_port(FIRST_FREE_PORT);

        assert_eq!(system.bind_port().unwrap(), FIRST_FREE_PORT + 1);
    }

    #[test]
    fn connect_is_refused_on_a_port_a_bind_handed_out() {
        let system = SystemMock::new();

        // A bind only reserves the number, so nothing answers on it until
        // whoever asked for it starts listening.
        let port = system.bind_port().unwrap();

        assert!(system.connect_port(port, Duration::from_secs(1)).is_err());
    }
}
