use std::io::stdout;
use std::io::Write;
use std::time::Instant;

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

    pub fn update(&self) {
        let mut stdout = stdout();
        print!(
            "\r{} {}",
            self.message,
            self.start
                .map(|start| format!("({:.1?}s)", start.elapsed().as_secs_f32()))
                .unwrap_or_default()
        );
        stdout.flush().ok();
    }

    pub fn done(&self) {
        println!();
    }
}
