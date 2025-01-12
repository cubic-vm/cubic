use crate::error::Error;

use libc;

use std::io::{self, prelude::*};
use std::mem;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

const TIOCGWINSZ: libc::c_ulong = 0x5413;

pub struct Terminal {
    threads: Vec<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

fn spawn_stdin_thread(sender: Sender<u8>, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8];
        while running.load(Ordering::Relaxed) {
            if io::stdin().read(&mut buffer).is_ok() {
                running.store(buffer[0] != 0x17, Ordering::Relaxed);
                sender.send(buffer[0]).ok();
            }
        }
    })
}

fn spawn_stream_thread(
    mut stream: UnixStream,
    receiver: Receiver<u8>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut termios_original: libc::termios;
        unsafe {
            termios_original = mem::zeroed();
            libc::tcgetattr(0, &mut termios_original);
            let mut termios = mem::zeroed();
            libc::tcgetattr(0, &mut termios);
            termios.c_lflag &= !libc::ICANON;
            termios.c_lflag &= !libc::ECHO;
            termios.c_lflag &= !libc::ISIG;
            termios.c_cc[libc::VMIN] = 1;
            termios.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(0, libc::TCSANOW, &termios);
        }

        let buf = &mut [0u8; 10];
        let mut out = std::io::stdout();

        while running.load(Ordering::Relaxed) {
            while let Ok(input) = receiver.try_recv() {
                stream.write_all(&[input]).ok();
            }
            stream.flush().ok();

            while let Ok(size) = stream.read(buf) {
                out.write_all(&buf[..size]).ok();
            }
            out.flush().ok();

            thread::sleep(Duration::from_millis(10));
        }

        stream.shutdown(Shutdown::Both).ok();
        out.write_all("\n".as_bytes()).ok();
        out.flush().ok();

        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &termios_original);
        }
    })
}

impl Terminal {
    pub fn open(path: &str) -> Result<Self, Error> {
        UnixStream::connect(path)
            .map(|stream| {
                stream.set_nonblocking(true).ok();
                stream
                    .set_read_timeout(Some(Duration::from_millis(10)))
                    .ok();
                let running = Arc::new(AtomicBool::new(true));
                let (tx, rx) = mpsc::channel::<u8>();
                Terminal {
                    threads: vec![
                        spawn_stdin_thread(tx, running.clone()),
                        spawn_stream_thread(stream, rx, running.clone()),
                    ],
                    running,
                }
            })
            .map_err(|_| Error::CannotOpenTerminal(path.to_string()))
    }

    pub fn is_running(&mut self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst)
    }

    pub fn get_term_size(&self) -> Option<(isize, isize)> {
        let winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            if libc::ioctl(libc::STDOUT_FILENO, TIOCGWINSZ, &winsize) == 0 {
                Some((winsize.ws_col as isize, winsize.ws_row as isize))
            } else {
                None
            }
        }
    }

    pub fn wait(&mut self) {
        while let Some(thread) = self.threads.pop() {
            thread.join().ok();
        }
    }
}
