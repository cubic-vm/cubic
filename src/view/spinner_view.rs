use std::cmp::max;
use std::io::{stdout, Write};
use std::sync::mpsc::{self, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time;

const SPINNER_VIEW_CHARS: &[char] = &['-', '\\', '|', '/'];

pub struct SpinnerView {
    channel: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl SpinnerView {
    pub fn new(text: &'static str) -> Self {
        let (tx, rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            let mut index = 0;
            let mut line_length = 0;
            let start = time::Instant::now();
            while let Err(TryRecvError::Empty) = rx.try_recv() {
                let text = format!(
                    "{}{}.. {} ({:.1?}s)",
                    if index > 0 { "\r" } else { "" },
                    &text,
                    SPINNER_VIEW_CHARS[index % SPINNER_VIEW_CHARS.len()],
                    start.elapsed().as_secs_f32()
                );

                line_length = max(line_length, text.len());
                print!("{text}");
                stdout().flush().ok();
                index += 1;
                thread::sleep(time::Duration::from_millis(100));
            }
            print!("\r{: ^width$}\r", "", width = line_length);
            stdout().flush().ok();
        });

        SpinnerView {
            channel: tx,
            thread: Some(thread),
        }
    }

    pub fn stop(&mut self) {
        self.channel.send(()).ok();
        self.thread.take().map(|t| t.join());
    }
}

impl Drop for SpinnerView {
    fn drop(&mut self) {
        self.stop();
    }
}
