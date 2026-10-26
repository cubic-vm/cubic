use crate::platform::{Stream, System};
use crate::view::Styler;
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::style::Color;
use crossterm::terminal::{Clear, ClearType};
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
fn enable_ansi_support() {
    crossterm::ansi_support::supports_ansi();
}

#[cfg(not(windows))]
fn enable_ansi_support() {}

pub struct Console {
    is_tty: bool,
    unicode: bool,
    stdout_styler: Styler,
    stderr_styler: Styler,
    verbosity: AtomicI8,
    // Only flipped inside set_paused, between building the clear and the draw.
    paused: AtomicBool,
    // Each frame line must fit the terminal width so one line takes one row.
    frame: Mutex<String>,
    system: Arc<dyn System>,
}

impl Console {
    const DEBUG_VERBOSITY: i8 = 1;
    const INFO_VERBOSITY: i8 = 0;
    const WARN_VERBOSITY: i8 = -1;

    pub fn new(system: Arc<dyn System>) -> Arc<Self> {
        enable_ansi_support();
        let stdout_tty = system.is_terminal(Stream::Stdout);
        let stderr_tty = system.is_terminal(Stream::Stderr);
        let color = system.read_env_var("NO_COLOR").is_none();
        Arc::new(Self {
            is_tty: stdout_tty,
            unicode: Self::detect_unicode(system.as_ref()),
            stdout_styler: Styler::new(color && stdout_tty),
            stderr_styler: Styler::new(color && stderr_tty),
            verbosity: AtomicI8::new(0),
            paused: AtomicBool::new(false),
            frame: Mutex::new(String::new()),
            system,
        })
    }

    pub fn has_unicode(&self) -> bool {
        self.unicode
    }

    fn detect_unicode(system: &dyn System) -> bool {
        let read = |key: &str| system.read_env_var(key).filter(|value| !value.is_empty());
        if ["linux", "dumb"].contains(&read("TERM").unwrap_or_default().as_str()) {
            return false;
        }
        let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
            .iter()
            .find_map(|var| read(var))
            .unwrap_or_default()
            .to_uppercase();
        read("WT_SESSION").is_some() || locale.contains("UTF-8") || locale.contains("UTF8")
    }

    fn get_verbosity(&self) -> i8 {
        self.verbosity.load(Ordering::Relaxed)
    }

    pub fn is_enabled(&self) -> bool {
        self.is_tty && self.get_verbosity() >= Self::INFO_VERBOSITY
    }

    fn redraw(&self, old: &str, new: &str) -> String {
        if !self.is_enabled() || self.paused.load(Ordering::Relaxed) {
            return String::new();
        }
        let mut out = String::new();
        let rows = old.lines().count();
        if rows > 1 {
            out.push_str(&MoveUp((rows - 1) as u16).to_string());
        }
        out.push_str(&format!(
            "{}{}{new}",
            MoveToColumn(0),
            Clear(ClearType::FromCursorDown)
        ));
        out
    }

    fn write(&self, text: &str) {
        self.system.print(Stream::Stdout, text);
        self.system.flush(Stream::Stdout);
    }

    pub fn update_animation(&self, frame: &str, width: usize) {
        let mut current = self.frame.lock().unwrap();
        let frame = frame
            .split('\n')
            .map(|line| line.chars().take(width).collect())
            .collect::<Vec<String>>()
            .join("\n");
        self.write(&self.redraw(&current, &frame));
        *current = frame;
    }

    pub fn clear_animation(&self) {
        self.update_animation("", 0);
    }

    fn set_paused(&self, paused: bool) {
        let frame = self.frame.lock().unwrap();
        let clear = self.redraw(&frame, "");
        self.paused.store(paused, Ordering::Relaxed);
        self.write(&(clear + &self.redraw("", &frame)));
    }

    fn emit(&self, stream: Stream, msg: &str, style: Option<(&str, Color)>) {
        let text = match style {
            Some((label, c)) => {
                // Align follow-up lines with the text after the label.
                let indent = format!("\n{}", " ".repeat(label.len() + 1));
                format!(
                    "{} {}",
                    self.get_styler(stream).bold_color(label, c),
                    msg.replace('\n', &indent)
                )
            }
            None => msg.to_string(),
        };
        let frame = self.frame.lock().unwrap();
        let clear = self.redraw(&frame, "");
        let draw = self.redraw("", &frame);
        if matches!(stream, Stream::Stdout) {
            self.write(&format!("{clear}{text}\n{draw}"));
        } else {
            // stderr can't share the stdout write, so the frame is drawn around it.
            self.write(&clear);
            self.system.println(stream, &text);
            self.write(&draw);
        }
    }

    pub fn get_styler(&self, stream: Stream) -> Styler {
        match stream {
            Stream::Stdout => self.stdout_styler,
            Stream::Stderr => self.stderr_styler,
        }
    }

    pub fn set_verbosity(&self, verbosity: i8) {
        self.verbosity.store(verbosity, Ordering::Relaxed);
    }

    pub fn print(&self, msg: &str) {
        self.emit(Stream::Stdout, msg, None);
    }

    pub fn debug(&self, msg: &str) {
        if self.get_verbosity() >= Self::DEBUG_VERBOSITY {
            self.emit(Stream::Stdout, msg, Some(("debug:", Color::Green)));
        }
    }

    pub fn info(&self, msg: &str) {
        if self.get_verbosity() >= Self::INFO_VERBOSITY {
            self.emit(Stream::Stdout, msg, Some(("info:", Color::Blue)));
        }
    }

    pub fn warn(&self, msg: &str) {
        if self.get_verbosity() >= Self::WARN_VERBOSITY {
            self.emit(Stream::Stderr, msg, Some(("warn:", Color::Yellow)));
        }
    }

    pub fn error(&self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("error:", Color::Red)));
    }

    pub fn flush(&self) {
        self.system.flush(Stream::Stdout);
        self.system.flush(Stream::Stderr);
    }

    pub fn get_geometry(&self) -> Option<(u32, u32)> {
        self.system.get_size()
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
    fn test_each_frame_fits_the_width_and_clears_the_one_before_it() {
        let system = Arc::new(SystemMock::new().set_terminal(true));
        let console = Console::new(Arc::clone(&system) as Arc<dyn System>);

        console.update_animation(&"x".repeat(100), 80);
        console.update_animation("second", 80);
        console.clear_animation();

        let home = MoveToColumn(0).to_string();
        let clear = Clear(ClearType::FromCursorDown).to_string();
        assert_eq!(
            system.get_output(),
            format!(
                "{home}{clear}{}{home}{clear}second{home}{clear}",
                "x".repeat(80)
            )
        );
    }

    #[test]
    fn test_shorter_frame_clears_the_lines_left_over() {
        let system = Arc::new(SystemMock::new().set_terminal(true));
        let console = Console::new(Arc::clone(&system) as Arc<dyn System>);

        console.update_animation("a\nb\nc", 80);
        console.update_animation("d", 80);

        let home = MoveToColumn(0).to_string();
        let clear = Clear(ClearType::FromCursorDown).to_string();
        let up = MoveUp(2).to_string();
        assert_eq!(
            system.get_output(),
            format!("{home}{clear}a\nb\nc{up}{home}{clear}d")
        );
    }

    #[test]
    fn test_unicode_follows_the_terminal_and_the_locale() {
        let detect = |vars: &[(&str, &str)]| {
            let system = vars
                .iter()
                .fold(SystemMock::new(), |system, (k, v)| system.add_env_var(k, v));
            Console::new(Arc::new(system)).has_unicode()
        };

        assert!(detect(&[("LANG", "en_US.UTF-8")]));
        assert!(detect(&[("LC_CTYPE", "de_CH.utf8")]));
        assert!(detect(&[("WT_SESSION", "1")]));
        assert!(!detect(&[]));
        assert!(!detect(&[("LC_ALL", "C"), ("LANG", "en_US.UTF-8")]));
        assert!(!detect(&[("TERM", "linux"), ("LANG", "en_US.UTF-8")]));
        assert!(!detect(&[("TERM", "dumb"), ("LANG", "en_US.UTF-8")]));
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
    fn test_styling_on_a_terminal_unless_no_color_is_set() {
        let styled = |system: SystemMock| {
            let console = Console::new(Arc::new(system));
            console.get_styler(Stream::Stdout).bold("x") != "x"
        };

        assert!(styled(SystemMock::new().set_terminal(true)));
        assert!(!styled(SystemMock::new()));
        assert!(!styled(
            SystemMock::new()
                .set_terminal(true)
                .add_env_var("NO_COLOR", "1")
        ));
    }
}
