use crate::commands::Verbosity;
use crate::platform::{Stream, System};
use crate::view::Animation;
use crossterm::QueueableCommand;
use crossterm::cursor::MoveToColumn;
use crossterm::style::{Attribute, Color, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use std::io::{Stdout, Write, stdout};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const ANIMATION_TICK_MS: u64 = 100;

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
    muted: bool,
    shutdown: bool,
}

pub struct Console<'a> {
    verbosity: Verbosity,
    is_tty: bool,
    state: Arc<AnimationState>,
    thread: Option<JoinHandle<()>>,
    system: &'a dyn System,
}

impl<'a> Console<'a> {
    pub fn new(system: &'a dyn System) -> Self {
        enable_ansi_support();
        Self {
            verbosity: Verbosity::new(false, false),
            is_tty: system.is_terminal(Stream::Stdout),
            state: Arc::new(AnimationState {
                inner: Mutex::new(AnimationInner {
                    animation: None,
                    muted: false,
                    shutdown: false,
                }),
                signal: Condvar::new(),
            }),
            thread: None,
            system,
        }
    }

    fn is_no_color(&self) -> bool {
        self.system.read_env_var("NO_COLOR").is_some()
    }

    // Print a message that coexists with a running animation. While an
    // animation is playing the render thread is held off (it only writes while
    // holding the state lock): clear its line, print the message, then redraw a
    // fresh frame immediately so the live line stays just below the output.
    fn emit(&self, stream: Stream, msg: &str, style: Option<(&str, Color)>) {
        let no_color = self.is_no_color();
        let enabled = style.is_some() && self.system.is_terminal(stream) && !no_color;
        let text = match style {
            Some((label, color)) => format!("{} {msg}", colorize(label, color, enabled)),
            None => msg.to_string(),
        };

        let inner = self.state.inner.lock().unwrap();
        let animation = inner.animation.clone();
        let mut stdout = stdout();
        if animation.is_some() {
            AnimationState::clear_line(&mut stdout);
        }
        self.system.println(stream, &text);
        if let Some(animation) = animation {
            AnimationState::draw_frame(&mut stdout, &animation);
        }
    }

    // Mute the animation render thread and clear its line so a prompt can
    // hold it. Pairs with unmute().
    fn mute(&self) {
        let mut inner = self.state.inner.lock().unwrap();
        if inner.animation.is_some() {
            inner.muted = true;
            AnimationState::clear_line(&mut stdout());
        }
    }

    // Unmute the animation render thread and redraw its current frame.
    fn unmute(&self) {
        let mut inner = self.state.inner.lock().unwrap();
        if let Some(animation) = inner.animation.clone() {
            inner.muted = false;
            AnimationState::draw_frame(&mut stdout(), &animation);
        }
    }

    pub fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    pub fn print(&mut self, msg: &str) {
        self.emit(Stream::Stdout, msg, None);
    }

    pub fn debug(&mut self, msg: &str) {
        if self.verbosity.is_verbose() {
            self.emit(Stream::Stdout, msg, Some(("debug:", Color::Green)));
        }
    }

    pub fn info(&mut self, msg: &str) {
        if !self.verbosity.is_quiet() {
            self.emit(Stream::Stdout, msg, Some(("info:", Color::Blue)));
        }
    }

    pub fn warn(&mut self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("warn:", Color::Yellow)));
    }

    pub fn error(&mut self, msg: &str) {
        self.emit(Stream::Stderr, msg, Some(("error:", Color::Red)));
    }

    pub fn get_geometry(&self) -> Option<(u32, u32)> {
        crossterm::terminal::size()
            .map(|(w, h)| (w as u32, h as u32))
            .ok()
    }

    pub fn prompt(&mut self, text: &str) -> String {
        self.mute();
        self.system.print(Stream::Stdout, text);
        self.system.flush(Stream::Stdout);
        let reply = self.system.read_input();
        self.unmute();
        reply
    }

    pub fn prompt_secret(&mut self, text: &str) -> Result<String, ()> {
        self.mute();
        self.system.print(Stream::Stdout, text);
        self.system.flush(Stream::Stdout);
        let result = self.system.read_secret();
        self.unmute();
        result
    }

    pub fn raw_mode(&mut self) {
        self.system.raw_mode();
    }

    pub fn reset(&mut self) {
        self.system.reset();
    }

    pub fn play(&mut self, animation: Arc<Mutex<dyn Animation>>) {
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

    pub fn stop(&mut self) {
        let mut inner = self.state.inner.lock().unwrap();
        if inner.animation.take().is_some() {
            AnimationState::clear_line(&mut stdout());
        }
        self.state.signal.notify_all();
    }
}

impl AnimationState {
    // One reusable thread renders the current animation. It parks on the
    // condvar while idle and ticks every ANIMATION_TICK_MS while an animation
    // is set, skipping the draw while muted so prompt() can hold the line.
    // The state lock is held during a render so that stop() can clear the
    // line without racing a write.
    fn run(self: Arc<Self>) {
        let mut stdout = stdout();
        let mut inner = self.inner.lock().unwrap();
        while !inner.shutdown {
            match inner.animation.clone() {
                None => {
                    inner = self.signal.wait(inner).unwrap();
                }
                Some(animation) => {
                    if !inner.muted {
                        Self::draw_frame(&mut stdout, &animation);
                    }
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

impl Drop for Console<'_> {
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
