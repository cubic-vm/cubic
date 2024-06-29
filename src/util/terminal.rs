use crate::error::Error;
use crate::util;
use libc;
use std::fs;
use std::io::prelude::*;

use std::mem;
use std::net::Shutdown;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process;
use std::thread::{self};

pub struct Terminal {
    stream: UnixStream,
}

impl Terminal {
    pub fn open(path: &str) -> Result<Self, Error> {
        let pid_file = format!("{path}.pid");
        let is_used = fs::read_to_string(&pid_file)
            .map(|pid| Path::new(&format!("/proc/{pid}")).exists())
            .unwrap_or(false);

        if is_used {
            Err(Error::CannotOpenTerminal(path.to_string()))
        } else {
            util::write_file(&pid_file, process::id().to_string().as_bytes())?;
            UnixStream::connect(path)
                .map(|stream| Terminal { stream })
                .map_err(|_| Error::CannotOpenTerminal(path.to_string()))
        }
    }

    pub fn run(&mut self) {
        let mut stream2 = self.stream.try_clone().unwrap();
        stream2.write_all("\n".as_bytes()).ok();

        unsafe {
            let mut termios_original: libc::termios = mem::zeroed();
            libc::tcgetattr(0, &mut termios_original);
            let mut termios = mem::zeroed();
            libc::tcgetattr(0, &mut termios);
            termios.c_lflag &= !libc::ICANON;
            termios.c_lflag &= !libc::ECHO;
            termios.c_lflag &= !libc::ISIG;
            termios.c_cc[libc::VMIN] = 1;
            termios.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(0, libc::TCSANOW, &termios);

            thread::spawn(move || {
                std::io::copy(&mut stream2, &mut fs::File::from_raw_fd(1)).ok();
            });

            for byte in std::io::stdin().bytes().flatten() {
                self.stream.write_all(&[byte]).ok();
                if byte == 0x4 {
                    break;
                }
            }
            self.stream.shutdown(Shutdown::Both).ok();
            libc::tcsetattr(0, libc::TCSANOW, &termios_original);
            println!();
        }
    }
}
