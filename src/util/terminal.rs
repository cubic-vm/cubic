use crate::error::{Error, Result};

use std::io::{self, Read, Write};
use std::net::TcpStream;
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

fn spawn_stream_thread<S: Read + Write + Send + 'static>(
    mut stream: S,
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

        drop(stream);
        out.write_all(b"\n").ok();
        out.flush().ok();
    })
}

impl Terminal {
    pub fn open(port: u16) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .map_err(|_| Error::CannotOpenTerminal(port.to_string()))?;
        stream.set_nonblocking(true).ok();
        stream
            .set_read_timeout(Some(Duration::from_millis(10)))
            .ok();

        let running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel::<u8>();
        Ok(Terminal {
            threads: vec![
                spawn_stdin_thread(tx, running.clone()),
                spawn_stream_thread(stream, rx, running.clone()),
            ],
        })
    }

    pub fn wait(&mut self) {
        while let Some(thread) = self.threads.pop() {
            thread.join().ok();
        }
    }
}
