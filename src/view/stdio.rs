use crate::commands::Verbosity;
use crate::view::{Animation, Console};
use crossterm::QueueableCommand;
use crossterm::cursor::MoveToColumn;
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use std::io::{IsTerminal, Stdout, Write, stderr, stdout};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const ANIMATION_TICK_MS: u64 = 100;

fn is_no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

// On Windows, ANSI escape codes are only interpreted once virtual terminal
// processing is turned on for the console; this call is a no-op elsewhere.
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

struct AnimationState {
    inner: Mutex<AnimationInner>,
    signal: Condvar,
}

struct AnimationInner {
    animation: Option<Arc<Mutex<dyn Animation>>>,
    shutdown: bool,
}

enum Stream {
    Stdout,
    Stderr,
}

impl Stream {
    fn is_terminal(&self) -> bool {
        match self {
            Stream::Stdout => stdout().is_terminal(),
            Stream::Stderr => stderr().is_terminal(),
        }
    }

    fn print(self, msg: &str, style: Option<(&str, Color)>) {
        let enabled = style.is_some() && self.is_terminal() && !is_no_color();
        let text = match style {
            Some((label, color)) => format!("{} {msg}", colorize(label, color, enabled)),
            None => msg.to_string(),
        };
        match self {
            Stream::Stdout => println!("{text}"),
            Stream::Stderr => eprintln!("{text}"),
        }
    }
}

pub struct Stdio {
    verbosity: Verbosity,
    is_tty: bool,
    state: Arc<AnimationState>,
    thread: Option<JoinHandle<()>>,
}

impl Stdio {
    pub fn new() -> Self {
        enable_ansi_support();
        Self {
            verbosity: Verbosity::new(false, false),
            is_tty: stdout().is_terminal(),
            state: Arc::new(AnimationState {
                inner: Mutex::new(AnimationInner {
                    animation: None,
                    shutdown: false,
                }),
                signal: Condvar::new(),
            }),
            thread: None,
        }
    }

    // Print a message that coexists with a running animation. While an
    // animation is playing the render thread is held off (it only writes while
    // holding the state lock): clear its line, print the message, then redraw a
    // fresh frame immediately so the live line stays just below the output.
    fn emit(&self, stream: Stream, msg: &str, style: Option<(&str, Color)>) {
        let inner = self.state.inner.lock().unwrap();
        let animation = inner.animation.clone();
        let mut stdout = stdout();
        if animation.is_some() {
            AnimationState::clear_line(&mut stdout);
        }
        stream.print(msg, style);
        if let Some(animation) = animation {
            AnimationState::draw_frame(&mut stdout, &animation);
        }
    }
}

impl AnimationState {
    // One reusable thread renders the current animation. It parks on the
    // condvar while idle and ticks every ANIMATION_TICK_MS while an animation
    // is set. The state lock is held during a render so that stop() can clear
    // the line without racing a write.
    fn run(self: Arc<Self>) {
        let mut stdout = stdout();
        let mut inner = self.inner.lock().unwrap();
        while !inner.shutdown {
            match inner.animation.clone() {
                None => {
                    inner = self.signal.wait(inner).unwrap();
                }
                Some(animation) => {
                    Self::draw_frame(&mut stdout, &animation);
                    let timeout = Duration::from_millis(ANIMATION_TICK_MS);
                    inner = self.signal.wait_timeout(inner, timeout).unwrap().0;
                }
            }
        }
    }

    fn draw_frame(stdout: &mut Stdout, animation: &Arc<Mutex<dyn Animation>>) {
        let width = crossterm::terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80);
        let line = animation.lock().unwrap().render(width);
        Self::draw_line(stdout, &line);
    }

    fn draw_line(stdout: &mut Stdout, line: &str) {
        stdout
            .queue(MoveToColumn(0))
            .and_then(|out| out.queue(Clear(ClearType::CurrentLine)))
            .and_then(|out| out.queue(Print(line)))
            .and_then(|out| out.flush())
            .ok();
    }

    fn clear_line(stdout: &mut Stdout) {
        Self::draw_line(stdout, "");
    }
}

impl Console for Stdio {
    fn get_verbosity(&mut self) -> Verbosity {
        self.verbosity
    }

    fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    fn print(&mut self, msg: &str) {
        self.emit(Stream::Stdout, msg, None);
    }

    fn info(&mut self, msg: &str) {
        if !self.verbosity.is_quiet() {
            self.emit(Stream::Stdout, msg, Some(("info:", Color::Blue)));
        }
    }

    fn warn(&mut self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("warn:", Color::Yellow)));
    }

    fn error(&mut self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("error:", Color::Red)));
    }

    fn get_geometry(&self) -> Option<(u32, u32)> {
        crossterm::terminal::size()
            .map(|(w, h)| (w as u32, h as u32))
            .ok()
    }

    fn raw_mode(&mut self) {
        crossterm::terminal::enable_raw_mode().ok();
    }

    fn reset(&mut self) {
        crossterm::terminal::disable_raw_mode().ok();
    }

    fn play(&mut self, animation: Arc<Mutex<dyn Animation>>) {
        if self.verbosity.is_quiet() || !self.is_tty {
            return;
        }

        if self.thread.is_none() {
            let state = Arc::clone(&self.state);
            self.thread = Some(thread::spawn(move || state.run()));
        }

        let mut inner = self.state.inner.lock().unwrap();
        inner.animation = Some(animation);
        self.state.signal.notify_all();
    }

    fn stop(&mut self) {
        let mut inner = self.state.inner.lock().unwrap();
        if inner.animation.take().is_some() {
            AnimationState::clear_line(&mut stdout());
        }
        self.state.signal.notify_all();
    }
}

impl Drop for Stdio {
    fn drop(&mut self) {
        self.stop();
        self.state.inner.lock().unwrap().shutdown = true;
        self.state.signal.notify_all();
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
