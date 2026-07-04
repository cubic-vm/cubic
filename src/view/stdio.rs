use crate::commands::Verbosity;
use crate::view::{Animation, Console};
use crossterm::QueueableCommand;
use crossterm::cursor::MoveToColumn;
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use std::io::{IsTerminal, Stdout, Write, stdout};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const ANIMATION_TICK_MS: u64 = 100;

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
    fn print(self, msg: &str) {
        match self {
            Stream::Stdout => println!("{msg}"),
            Stream::Stderr => eprintln!("{msg}"),
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
    fn emit(&self, stream: Stream, msg: &str) {
        let inner = self.state.inner.lock().unwrap();
        let animation = inner.animation.clone();
        let mut stdout = stdout();
        if animation.is_some() {
            AnimationState::clear_line(&mut stdout);
        }
        stream.print(msg);
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
        let line = animation.lock().unwrap().render();
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

    fn info(&mut self, msg: &str) {
        if !self.verbosity.is_quiet() {
            self.emit(Stream::Stdout, msg);
        }
    }

    fn error(&mut self, msg: &str) {
        self.emit(Stream::Stderr, msg);
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
