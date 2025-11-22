use crate::error::Error;

use std::io::{self, prelude::*};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
};
use std::thread;
use std::time::Duration;

pub struct Terminal {
    threads: Vec<thread::JoinHandle<()>>,
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
                }
            })
            .map_err(|_| Error::CannotOpenTerminal(path.to_string()))
    }

    pub fn wait(&mut self) {
        while let Some(thread) = self.threads.pop() {
            thread.join().ok();
        }
    }
}
