use std::io::stdout;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct TimerView {
    message: String,
    start: Option<Instant>,
}

impl TimerView {
    pub fn new(message: &str) -> Self {
        TimerView {
            message: message.to_string(),
            start: Some(Instant::now()),
        }
    }

    pub fn run(&self, is_done: &mut impl FnMut() -> bool) {
        let mut stdout = stdout();

        while !is_done() {
            print!(
                "\r{} {}",
                self.message,
                self.start
                    .map(|start| format!("({:.1?}s)", start.elapsed().as_secs_f32()))
                    .unwrap_or_default()
            );
            stdout.flush().ok();
            sleep(Duration::from_millis(10));
        }
        println!();
    }
}
