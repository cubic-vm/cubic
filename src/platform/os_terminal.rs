use crate::platform::{OsSystem, Stream, Terminal};
use std::io::{IsTerminal, Read, Write, stderr, stdin, stdout};

impl OsSystem {
    // A closed pipe must not fail the command.
    fn write_stream(mut out: impl Write, text: &str) {
        out.write_all(text.as_bytes()).ok();
        out.flush().ok();
    }
}

impl Terminal for OsSystem {
    fn write(&self, stream: Stream, text: &str) {
        match stream {
            Stream::Stdout => OsSystem::write_stream(stdout().lock(), text),
            Stream::Stderr => OsSystem::write_stream(stderr().lock(), text),
        }
    }

    fn is_terminal(&self, stream: Stream) -> bool {
        match stream {
            Stream::Stdout => stdout().is_terminal(),
            Stream::Stderr => stderr().is_terminal(),
        }
    }

    fn is_stdin_terminal(&self) -> bool {
        stdin().is_terminal()
    }

    fn get_size(&self) -> Option<(u32, u32)> {
        crossterm::terminal::size()
            .map(|(w, h)| (w as u32, h as u32))
            .ok()
    }

    fn read_input(&self, masked: bool) -> std::result::Result<String, ()> {
        if !masked {
            let mut reply = String::new();
            stdin().read_line(&mut reply).ok();
            return Ok(reply.trim().to_string());
        }

        self.set_raw_mode(true);
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
                    self.set_raw_mode(false);
                    self.write(Stream::Stdout, "\n");
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
        self.set_raw_mode(false);
        self.write(Stream::Stdout, "\r\n");

        if failed || !pending.is_empty() {
            return Err(());
        }

        Ok(password)
    }

    fn set_raw_mode(&self, enabled: bool) {
        if enabled {
            crossterm::terminal::enable_raw_mode().ok();
        } else {
            crossterm::terminal::disable_raw_mode().ok();
        }
    }
}
