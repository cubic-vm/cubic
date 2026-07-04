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
    fn render(&mut self, width: usize) -> String {
        let frame = SPINNER_CHARS[self.index % SPINNER_CHARS.len()];
        self.index += 1;
        let right = format!("({}) {}", self.duration, frame);
        let gap = width.saturating_sub(self.text.len() + right.len()).max(1);
        format!("{}{}{}", self.text, " ".repeat(gap), right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_left_spinner_right() {
        let mut spinner = Spinner::new("Cloning VM instance".to_string());
        let line = spinner.render(60);
        assert_eq!(line.len(), 60);
        assert!(line.starts_with("Cloning VM instance"));
        assert!(line.ends_with('-'));
    }

    #[test]
    fn test_narrow_width_never_panics() {
        let mut spinner = Spinner::new("Cloning VM instance".to_string());
        let line = spinner.render(4);
        assert!(line.starts_with("Cloning VM instance"));
    }
}
