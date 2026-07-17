use crate::platform::Stream;
use std::io::{IsTerminal, Read, Write, stderr, stdin, stdout};

pub trait System {
    fn read_env_var(&self, key: &str) -> Option<String>;

    fn print(&self, stream: Stream, msg: &str);
    fn println(&self, stream: Stream, msg: &str);
    fn flush(&self, stream: Stream);
    fn is_terminal(&self, stream: Stream) -> bool;

    fn read_input(&self) -> String;
    fn read_secret(&self) -> Result<String, ()>;

    fn raw_mode(&self);
    fn reset(&self);
}

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
    fn read_secret(&self) -> Result<String, ()> {
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
}
