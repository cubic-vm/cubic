use crate::view::Animation;
use std::time::{Duration, Instant};

const SPINNER_CHARS: &[char] = &['-', '\\', '|', '/'];

pub struct Spinner {
    text: String,
    start: Instant,
}

impl Spinner {
    pub fn new(text: String) -> Self {
        Self {
            text,
            start: Instant::now(),
        }
    }

    fn render_duration(&self, duration: Duration, width: usize) -> String {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        let tenth = duration.as_millis() / 100;
        let spinner = SPINNER_CHARS[(tenth % SPINNER_CHARS.len() as u128) as usize];
        let tenth = tenth % 10;
        let time = if minutes > 0 {
            format!("{minutes}m {seconds:02}.{tenth}s")
        } else {
            format!("{seconds}.{tenth}s")
        };

        let mut output = String::with_capacity(width);
        if seconds.is_multiple_of(2) {
            output.push('*');
        } else {
            output.push(' ');
        }
        output.push(' ');
        output.push_str(&self.text);
        output.push(' ');
        output.push(spinner);
        output.push_str(
            &" ".repeat(
                width
                    .saturating_sub(output.chars().count() + time.chars().count())
                    .max(1),
            ),
        );
        output.push_str(&time);
        output.chars().take(width).collect()
    }
}

impl Animation for Spinner {
    fn render(&mut self, width: usize) -> String {
        self.render_duration(self.start.elapsed(), width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frames() {
        let spinner = Spinner::new("Cloning foobar".to_string());
        let line_length = 25;

        let frame1 = spinner.render_duration(Duration::from_millis(0), line_length);
        assert_eq!(frame1.len(), line_length);
        assert_eq!(frame1, "* Cloning foobar -   0.0s");

        let frame2 = spinner.render_duration(Duration::from_millis(100), line_length);
        assert_eq!(frame2.len(), line_length);
        assert_eq!(frame2, "* Cloning foobar \\   0.1s");

        let frame3 = spinner.render_duration(Duration::from_millis(200), line_length);
        assert_eq!(frame3.len(), line_length);
        assert_eq!(frame3, "* Cloning foobar |   0.2s");

        let frame4 = spinner.render_duration(Duration::from_millis(300), line_length);
        assert_eq!(frame4.len(), line_length);
        assert_eq!(frame4, "* Cloning foobar /   0.3s");
    }

    #[test]
    fn test_bullet_point() {
        let spinner = Spinner::new("Starting myinstance".to_string());
        let line_length = 30;

        let frame1 = spinner.render_duration(Duration::from_millis(0), line_length);
        assert_eq!(frame1.len(), line_length);
        assert_eq!(frame1, "* Starting myinstance -   0.0s");

        let frame2 = spinner.render_duration(Duration::from_millis(500), line_length);
        assert_eq!(frame2.len(), line_length);
        assert_eq!(frame2, "* Starting myinstance \\   0.5s");

        let frame3 = spinner.render_duration(Duration::from_millis(1000), line_length);
        assert_eq!(frame3.len(), line_length);
        assert_eq!(frame3, "  Starting myinstance |   1.0s");

        let frame4 = spinner.render_duration(Duration::from_millis(1500), line_length);
        assert_eq!(frame4.len(), line_length);
        assert_eq!(frame4, "  Starting myinstance /   1.5s");

        let frame5 = spinner.render_duration(Duration::from_millis(2000), line_length);
        assert_eq!(frame5.len(), line_length);
        assert_eq!(frame5, "* Starting myinstance -   2.0s");
    }

    #[test]
    fn test_duration() {
        let spinner = Spinner::new("Stopping quickstart".to_string());
        let line_length = 35;

        let frame1 = spinner.render_duration(Duration::from_millis(1), line_length);
        assert_eq!(frame1, "* Stopping quickstart -        0.0s");

        let frame2 = spinner.render_duration(Duration::from_secs(1), line_length);
        assert_eq!(frame2, "  Stopping quickstart |        1.0s");

        let frame3 = spinner.render_duration(Duration::from_mins(1), line_length);
        assert_eq!(frame3, "* Stopping quickstart -    1m 00.0s");

        let frame4 = spinner.render_duration(Duration::from_millis(135432), line_length);
        assert_eq!(frame4, "  Stopping quickstart |    2m 15.4s");
    }

    #[test]
    fn test_resize() {
        let spinner = Spinner::new("Cloning foobar".to_string());

        let frame1 = spinner.render_duration(Duration::from_millis(1342), 40);
        assert_eq!(frame1.len(), 40);
        assert_eq!(frame1, "  Cloning foobar \\                  1.3s");

        let frame2 = spinner.render_duration(Duration::from_millis(1342), 20);
        assert_eq!(frame2.len(), 20);
        assert_eq!(frame2, "  Cloning foobar \\ 1");

        let frame3 = spinner.render_duration(Duration::from_millis(1342), 10);
        assert_eq!(frame3.len(), 10);
        assert_eq!(frame3, "  Cloning ");

        let frame4 = spinner.render_duration(Duration::from_millis(1342), 5);
        assert_eq!(frame4.len(), 5);
        assert_eq!(frame4, "  Clo");

        let frame5 = spinner.render_duration(Duration::from_millis(1342), 0);
        assert_eq!(frame5.len(), 0);
    }
}
