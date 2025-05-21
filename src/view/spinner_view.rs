use std::io::{stdout, Write};
use std::thread;
use std::time;

const SPINNER_VIEW_CHARS: &[char] = &['-', '\\', '|', '/'];

pub struct SpinnerView {
    text: String,
}

impl SpinnerView {
    pub fn new(text: &str) -> Self {
        SpinnerView {
            text: text.to_string(),
        }
    }

    pub fn run<T: std::marker::Send + 'static>(&mut self, f: fn() -> T) -> Option<T> {
        let thread = thread::spawn(f);
        let mut index = 0;

        while !thread.is_finished() {
            print!(
                "\r{}.. {}",
                &self.text,
                SPINNER_VIEW_CHARS[index % SPINNER_VIEW_CHARS.len()]
            );
            stdout().flush().ok();
            thread::sleep(time::Duration::from_millis(100));
            index += 1;
        }

        print!("\r");
        thread.join().ok()
    }
}
