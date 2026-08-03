use crate::error::{Error, Result};
use crate::platform::{ReadWrite, Stream, System};
use crate::util::SystemCommand;
use std::fs;
use std::io::{ErrorKind, IsTerminal, Read, Write, stderr, stdin, stdout};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::from_utf8;
use std::time::Duration;

#[derive(Default)]
pub struct OsSystem;

impl OsSystem {
    pub fn new() -> Self {
        Self
    }

    // Refreshes a single process so a lookup sees the current state of the
    // host. `sysinfo::System` is spelled out because the name collides with
    // the `System` trait implemented below.
    fn read_process_table(pid: u64) -> (sysinfo::System, sysinfo::Pid) {
        let sys_pid = sysinfo::Pid::from_u32(pid as u32);
        let mut system = sysinfo::System::new();
        system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);
        (system, sys_pid)
    }

    fn build_process(command: &SystemCommand) -> Command {
        let mut process = Command::new(command.get_program());
        process.args(command.get_args());
        for (key, value) in command.get_envs() {
            process.env(key, value);
        }
        process
    }

    // A missing binary is worth telling apart from a binary that refused to
    // start, since only the first one means the host lacks the tool.
    fn map_spawn_error(command: &SystemCommand, error: std::io::Error) -> Error {
        if error.kind() == ErrorKind::NotFound {
            return Error::SystemCommandNotFound(command.get_program().to_string());
        }

        Error::SystemCommandFailed(command.get_command(), error.to_string())
    }
}

// Detaches the child from the session of the parent, so it survives the shell
// that started cubic.
#[cfg(unix)]
fn detach_from_session() -> std::io::Result<()> {
    unsafe extern "C" {
        fn setsid() -> i32;
    }
    if unsafe { setsid() } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

impl System for OsSystem {
    fn read_env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn print(&self, stream: Stream, msg: &str) {
        match stream {
            Stream::Stdout => print!("{msg}"),
            Stream::Stderr => eprint!("{msg}"),
        }
    }

    fn println(&self, stream: Stream, msg: &str) {
        match stream {
            Stream::Stdout => println!("{msg}"),
            Stream::Stderr => eprintln!("{msg}"),
        }
    }

    fn flush(&self, stream: Stream) {
        match stream {
            Stream::Stdout => stdout().flush().ok(),
            Stream::Stderr => stderr().flush().ok(),
        };
    }

    fn is_terminal(&self, stream: Stream) -> bool {
        match stream {
            Stream::Stdout => stdout().is_terminal(),
            Stream::Stderr => stderr().is_terminal(),
        }
    }

    fn read_input(&self) -> String {
        let mut reply = String::new();
        stdin().read_line(&mut reply).unwrap();
        reply.trim().to_string()
    }

    // Reads a password character by character in raw mode, without echoing
    // input back to the terminal (not even as masking characters).
    fn read_secret(&self) -> std::result::Result<String, ()> {
        self.raw_mode();
        let mut password = String::new();
        let mut pending = Vec::new();
        let mut stdin = stdin();
        let mut failed = false;
        loop {
            let mut byte = [0u8];
            match stdin.read(&mut byte) {
                Ok(0) | Err(_) => {
                    failed = true;
                    break;
                }
                Ok(_) => {}
            }

            match byte[0] {
                // Ctrl+C
                0x03 => {
                    self.reset();
                    println!();
                    std::process::exit(1)
                }

                // Carriage return and line feed
                0x0A | 0x0D => break,

                // Backspace and delete
                0x08 | 0x7F => {
                    pending.clear();
                    password.pop();
                }

                byte => {
                    pending.push(byte);
                    match std::str::from_utf8(&pending) {
                        Ok(text) => {
                            password.push_str(text);
                            pending.clear();
                        }
                        Err(err) if err.error_len().is_some() => {
                            failed = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.reset();
        self.print(Stream::Stdout, "\r\n");
        self.flush(Stream::Stdout);

        if failed || !pending.is_empty() {
            return Err(());
        }

        Ok(password)
    }

    fn raw_mode(&self) {
        crossterm::terminal::enable_raw_mode().ok();
    }

    fn reset(&self) {
        crossterm::terminal::disable_raw_mode().ok();
    }

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
        fs::create_dir_all(path).map_err(|e| {
            Error::FS(format!(
                "Cannot create directory '{}' ({e})",
                path.display()
            ))
        })
    }

    fn create_writable_dir(&self, path: &Path) -> Result<()> {
        self.create_dir(path)?;

        let permission = fs::metadata(path)
            .map_err(|e| {
                Error::FS(format!(
                    "Cannot read directory metadata '{}' ({e})",
                    path.display()
                ))
            })?
            .permissions();

        if permission.readonly() {
            return Err(Error::FS(format!(
                "Cannot write directory '{}'",
                path.display()
            )));
        }

        Ok(())
    }

    fn remove_dir(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).map_err(|e| {
            Error::FS(format!(
                "Cannot remove directory '{}' ({e})",
                path.display()
            ))
        })
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        fs::read_dir(path)
            .map(|dir| dir.flatten().map(|entry| entry.path()).collect())
            .map_err(|e| Error::FS(format!("Cannot read directory '{}' ({e})", path.display())))
    }

    fn create_file(&self, path: &Path) -> Result<Box<dyn Write>> {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map(|f| Box::new(f) as Box<dyn Write>)
            .map_err(|e| Error::FS(format!("Cannot create file '{}' ({e})", path.display())))
    }

    fn open_file(&self, path: &Path) -> Result<Box<dyn Read>> {
        fs::File::open(path)
            .map(|f| Box::new(f) as Box<dyn Read>)
            .map_err(|e| Error::FS(format!("Cannot open file '{}' ({e})", path.display())))
    }

    fn read_file_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .map_err(|e| Error::FS(format!("Cannot read file '{}' ({e})", path.display())))
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        fs::write(path, contents)
            .map_err(|e| Error::FS(format!("Cannot write file '{}' ({e})", path.display())))
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
            .map_err(|e| Error::FS(format!("Cannot write file '{}' ({e})", path.display())))
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to).map_err(|e| {
            Error::FS(format!(
                "Cannot rename file from '{}' to '{}' ({e})",
                from.display(),
                to.display()
            ))
        })
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        fs::remove_file(path)
            .map_err(|e| Error::FS(format!("Cannot delete file '{}' ({e})", path.display())))
    }

    fn connect_port(&self, port: u16, timeout: Duration) -> Result<Box<dyn ReadWrite>> {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}")).map_err(Error::from)?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(Error::from)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(Error::from)?;
        Ok(Box::new(stream))
    }

    // The listener is dropped right away, so the port is only reserved for as
    // long as it takes the caller to hand it to whoever binds it for real.
    fn bind_port(&self) -> Result<u16> {
        TcpListener::bind("127.0.0.1:0")
            .map_err(|_| Error::NoPortAvailable)?
            .local_addr()
            .map(|addr| addr.port())
            .map_err(|_| Error::NoPortAvailable)
    }

    fn exists_process(&self, pid: u64) -> bool {
        let (system, sys_pid) = Self::read_process_table(pid);
        system.process(sys_pid).is_some()
    }

    fn kill_process(&self, pid: u64) -> Result<()> {
        let (system, sys_pid) = Self::read_process_table(pid);
        let process = system.process(sys_pid).ok_or(Error::ProcessNotFound(pid))?;

        process.kill().then_some(()).ok_or(Error::KillFailed(pid))
    }

    fn run_command(&self, command: &SystemCommand) -> Result<Vec<u8>> {
        Self::build_process(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| Self::map_spawn_error(command, e))
            .and_then(|out| {
                if out.status.success() {
                    Ok(out.stdout)
                } else {
                    Err(Error::SystemCommandFailed(
                        command.get_command(),
                        from_utf8(&out.stderr).unwrap_or_default().to_string(),
                    ))
                }
            })
    }

    fn spawn_command(&self, command: &SystemCommand) -> Result<()> {
        let mut process = Self::build_process(command);

        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt;
            process.pre_exec(detach_from_session);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            process.creation_flags(DETACHED_PROCESS);
        }

        let mut child = process
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Self::map_spawn_error(command, e))?;

        // Check immediately for instant failures, then again after a brief window
        // to catch startup errors (KVM permission denied, bad firmware, port conflicts).
        for ms in [0, 100] {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            if let Ok(Some(_)) = child.try_wait() {
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        let mut buf = Vec::new();
                        s.read_to_end(&mut buf).unwrap_or(0);
                        from_utf8(&buf).unwrap_or_default().to_owned()
                    })
                    .unwrap_or_default();
                return Err(Error::SystemCommandFailed(command.get_command(), stderr));
            }
        }

        // Reap the child when it eventually exits to avoid a zombie process.
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }

    fn get_total_memory(&self) -> u64 {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.total_memory()
    }

    fn get_available_memory(&self) -> u64 {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.available_memory()
    }

    // Counts logical processors, so a host with simultaneous multithreading
    // reports its thread count rather than its physical cores.
    fn get_cpu_count(&self) -> u16 {
        let mut system = sysinfo::System::new();
        system.refresh_cpu_all();
        system.cpus().len() as u16
    }
}
