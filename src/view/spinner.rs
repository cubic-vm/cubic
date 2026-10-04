use crate::view::Console;
use std::sync::Arc;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const SPINNER_CHARS: &[char] = &['-', '\\', '|', '/'];
const UNICODE_SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const TICK: Duration = Duration::from_millis(100);

// A running spinner. It ticks on its own thread and clears the line when the
// guard drops.
pub struct Spinner {
    signal: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
    console: Arc<Console>,
}

impl Spinner {
    pub fn new(console: Arc<Console>, text: String) -> Self {
        let (signal, thread) = if console.is_enabled() {
            let (tx, rx) = mpsc::channel();
            let console = Arc::clone(&console);
            let handle = thread::spawn(move || {
                let start = Instant::now();
                let unicode = console.has_unicode();
                loop {
                    let frame =
                        Self::render_frame(&text, start.elapsed(), console.width(), unicode);
                    console.update_animation(&frame);
                    match rx.recv_timeout(TICK) {
                        Err(RecvTimeoutError::Timeout) => continue,
                        _ => break,
                    }
                }
            });
            (Some(tx), Some(handle))
        } else {
            (None, None)
        };

        Self {
            signal,
            thread,
            console,
        }
    }

    fn render_frame(text: &str, duration: Duration, width: usize, unicode: bool) -> String {
        let chars = if unicode {
            UNICODE_SPINNER_CHARS
        } else {
            SPINNER_CHARS
        };
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        let tenth = duration.as_millis() / 100;
        let spinner = chars[(tenth % chars.len() as u128) as usize];
        let tenth = tenth % 10;
        let time = if minutes > 0 {
            format!("{minutes}m {seconds:02}.{tenth}s")
        } else {
            format!("{seconds}.{tenth}s")
        };

        let left = format!("{spinner} {text} ");
        let time_width = width.saturating_sub(left.chars().count());
        format!("{left}{time:>time_width$}")
    }

    // Also runs on drop, and is safe to call twice.
    pub fn stop(&mut self) {
        self.signal.take();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
        self.console.clear_animation();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frames() {
        let text = "Cloning foobar";
        let line_length = 25;

        let frame1 = Spinner::render_frame(text, Duration::from_millis(0), line_length, false);
        assert_eq!(frame1.len(), line_length);
        assert_eq!(frame1, "- Cloning foobar     0.0s");

        let frame2 = Spinner::render_frame(text, Duration::from_millis(100), line_length, false);
        assert_eq!(frame2.len(), line_length);
        assert_eq!(frame2, "\\ Cloning foobar     0.1s");

        let frame3 = Spinner::render_frame(text, Duration::from_millis(200), line_length, false);
        assert_eq!(frame3.len(), line_length);
        assert_eq!(frame3, "| Cloning foobar     0.2s");

        let frame4 = Spinner::render_frame(text, Duration::from_millis(300), line_length, false);
        assert_eq!(frame4.len(), line_length);
        assert_eq!(frame4, "/ Cloning foobar     0.3s");

        let unicode = Spinner::render_frame(text, Duration::from_millis(200), line_length, true);
        assert_eq!(unicode, "⠹ Cloning foobar     0.2s");
    }

    #[test]
    fn test_duration() {
        let text = "Stopping quickstart";
        let line_length = 35;

        let frame1 = Spinner::render_frame(text, Duration::from_millis(1), line_length, false);
        assert_eq!(frame1, "- Stopping quickstart          0.0s");

        let frame2 = Spinner::render_frame(text, Duration::from_secs(1), line_length, false);
        assert_eq!(frame2, "| Stopping quickstart          1.0s");

        let frame3 = Spinner::render_frame(text, Duration::from_mins(1), line_length, false);
        assert_eq!(frame3, "- Stopping quickstart      1m 00.0s");

        let frame4 = Spinner::render_frame(text, Duration::from_millis(135432), line_length, false);
        assert_eq!(frame4, "| Stopping quickstart      2m 15.4s");
    }

    #[test]
    fn test_resize() {
        let text = "Cloning foobar";

        let frame1 = Spinner::render_frame(text, Duration::from_millis(1342), 40, false);
        assert_eq!(frame1.len(), 40);
        assert_eq!(frame1, "\\ Cloning foobar                    1.3s");

        let frame2 = Spinner::render_frame(text, Duration::from_millis(1342), 10, false);
        assert_eq!(frame2, "\\ Cloning foobar 1.3s");
    }
}
