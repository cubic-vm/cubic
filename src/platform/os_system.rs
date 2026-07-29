use crate::error::{Error, Result};
use crate::platform::{Stream, System};
use std::fs;
use std::io::{IsTerminal, Read, Write, stderr, stdin, stdout};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct OsSystem;

impl OsSystem {
    pub fn new() -> Self {
        Self
    }
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
}
