use crate::view::{Animation, TimeDuration};

const SPINNER_CHARS: &[char] = &['-', '\\', '|', '/'];

pub struct Spinner {
    text: String,
    index: usize,
    duration: TimeDuration,
}

impl Spinner {
    pub fn new(text: String) -> Self {
        Self {
            text,
            index: 0,
            duration: TimeDuration::new(),
        }
    }
}

impl Animation for Spinner {
    fn render(&mut self) -> String {
        let frame = SPINNER_CHARS[self.index % SPINNER_CHARS.len()];
        self.index += 1;
        format!("{}.. {} ({})", self.text, frame, self.duration)
    }
}
