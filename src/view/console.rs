use crate::platform::{Stream, System};
use crate::view::Verbosity;
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
fn enable_ansi_support() {
    crossterm::ansi_support::supports_ansi();
}

#[cfg(not(windows))]
fn enable_ansi_support() {}

fn colorize(label: &str, color: Color, enabled: bool) -> String {
    if enabled {
        format!(
            "{}{}{label}{}",
            SetForegroundColor(color),
            SetAttribute(Attribute::Bold),
            SetAttribute(Attribute::Reset)
        )
    } else {
        label.to_string()
    }
}

pub struct Console {
    is_tty: bool,
    verbosity: Mutex<Verbosity>,
    // Only flipped inside redraw so a pause lands between the clear and the draw.
    paused: AtomicBool,
    // Each frame line must fit the terminal width so one line takes one row.
    frame: Mutex<String>,
    system: Arc<dyn System>,
}

impl Console {
    pub fn new(system: Arc<dyn System>) -> Arc<Self> {
        enable_ansi_support();
        let is_tty = system.is_terminal(Stream::Stdout);
        Arc::new(Self {
            is_tty,
            verbosity: Mutex::new(Verbosity::new(false, false)),
            paused: AtomicBool::new(false),
            frame: Mutex::new(String::new()),
            system,
        })
    }

    fn get_verbosity(&self) -> Verbosity {
        *self.verbosity.lock().unwrap()
    }

    pub fn is_enabled(&self) -> bool {
        self.is_tty && !self.get_verbosity().is_quiet()
    }

    fn is_visible(&self, frame: &str) -> bool {
        self.is_enabled() && !self.paused.load(Ordering::Relaxed) && !frame.is_empty()
    }

    fn clear_frame(&self, frame: &str) {
        if !self.is_visible(frame) {
            return;
        }
        let mut out = format!("{}{}", MoveToColumn(0), Clear(ClearType::CurrentLine));
        for _ in 1..frame.split('\n').count() {
            out.push_str(&format!("{}{}", MoveUp(1), Clear(ClearType::CurrentLine)));
        }
        self.write(&out);
    }

    fn draw_frame(&self, frame: &str) {
        if !self.is_visible(frame) {
            return;
        }
        self.write(&format!("{}{frame}", MoveToColumn(0)));
    }

    fn write(&self, text: &str) {
        self.system.print(Stream::Stdout, text);
        self.system.flush(Stream::Stdout);
    }

    pub fn update_animation(&self, frame: &str) {
        let mut current = self.frame.lock().unwrap();
        self.clear_frame(&current);
        *current = frame.to_string();
        self.draw_frame(&current);
    }

    pub fn clear_animation(&self) {
        self.update_animation("");
    }

    fn set_paused(&self, paused: bool) {
        let frame = self.frame.lock().unwrap();
        self.clear_frame(&frame);
        self.paused.store(paused, Ordering::Relaxed);
        self.draw_frame(&frame);
    }

    fn emit(&self, stream: Stream, msg: &str, style: Option<(&str, Color)>) {
        let color = style.is_some()
            && self.system.is_terminal(stream)
            && self.system.read_env_var("NO_COLOR").is_none();
        let text = match style {
            Some((label, c)) => {
                // Align follow-up lines with the text after the label.
                let indent = format!("\n{}", " ".repeat(label.len() + 1));
                format!(
                    "{} {}",
                    colorize(label, c, color),
                    msg.replace('\n', &indent)
                )
            }
            None => msg.to_string(),
        };
        let frame = self.frame.lock().unwrap();
        self.clear_frame(&frame);
        self.system.println(stream, &text);
        self.draw_frame(&frame);
    }

    pub fn set_verbosity(&self, verbosity: Verbosity) {
        *self.verbosity.lock().unwrap() = verbosity;
    }

    pub fn print(&self, msg: &str) {
        self.emit(Stream::Stdout, msg, None);
    }

    pub fn debug(&self, msg: &str) {
        if self.get_verbosity().is_verbose() {
            self.emit(Stream::Stdout, msg, Some(("debug:", Color::Green)));
        }
    }

    pub fn info(&self, msg: &str) {
        if !self.get_verbosity().is_quiet() {
            self.emit(Stream::Stdout, msg, Some(("info:", Color::Blue)));
        }
    }

    pub fn warn(&self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("warn:", Color::Yellow)));
    }

    pub fn error(&self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("error:", Color::Red)));
    }

    pub fn flush(&self) {
        self.system.flush(Stream::Stdout);
        self.system.flush(Stream::Stderr);
    }

    pub fn get_geometry(&self) -> Option<(u32, u32)> {
        crossterm::terminal::size()
            .map(|(w, h)| (w as u32, h as u32))
            .ok()
    }

    pub fn width(&self) -> usize {
        self.get_geometry().map_or(80, |(w, _)| w as usize)
    }

    pub fn prompt(&self, text: &str, masked: bool) -> Result<String, ()> {
        self.set_paused(true);
        self.system.print(Stream::Stdout, text);
        self.system.flush(Stream::Stdout);
        let reply = if masked {
            self.system.read_secret()
        } else {
            Ok(self.system.read_input())
        };
        self.set_paused(false);
        reply
    }

    pub fn raw_mode(&self) {
        self.system.raw_mode();
    }

    pub fn reset(&self) {
        self.system.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_each_frame_clears_the_one_before_it() {
        let system = Arc::new(SystemMock::new().set_terminal(true));
        let console = Console::new(Arc::clone(&system) as Arc<dyn System>);

        console.update_animation("first");
        console.update_animation("second");
        console.clear_animation();

        let home = MoveToColumn(0).to_string();
        let clear = format!("{home}{}", Clear(ClearType::CurrentLine));
        assert_eq!(
            system.get_output(),
            format!("{home}first{clear}{home}second{clear}")
        );
    }

    #[test]
    fn test_multiline_message_aligns_with_label() {
        let system = Arc::new(SystemMock::new());
        let console = Console::new(Arc::clone(&system) as Arc<dyn System>);

        console.warn("foo\n\nbar");
        console.error("foo\nbar");

        assert_eq!(
            system.get_output(),
            "warn: foo\n      \n      bar\nerror: foo\n       bar\n"
        );
    }

    #[test]
    fn test_colorize_wraps_label_when_enabled() {
        let text = colorize("error:", Color::Red, true);
        assert!(text.starts_with("\u{1b}["));
        assert!(text.contains("error:"));
        assert!(text.ends_with("\u{1b}[0m"));
    }

    #[test]
    fn test_colorize_leaves_label_unchanged_when_disabled() {
        assert_eq!(colorize("error:", Color::Red, false), "error:");
    }
}
