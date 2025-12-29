use crate::commands::Verbosity;
use crate::view::Console;
use std::io::{Stdout, stdout};
use termion::{
    self,
    raw::{IntoRawMode, RawTerminal},
};

pub struct Stdio {
    verbosity: Verbosity,
    raw_mode: Option<RawTerminal<Stdout>>,
}

impl Stdio {
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::new(false, false),
            raw_mode: None,
        }
    }
}

impl Console for Stdio {
    fn get_verbosity(&mut self) -> Verbosity {
        self.verbosity
    }

    fn set_verbosity(&mut self, verbosity: Verbosity) {
        self.verbosity = verbosity;
    }

    fn debug(&mut self, msg: &str) {
        if self.verbosity.is_verbose() {
            println!("{msg}");
        }
    }

    fn info(&mut self, msg: &str) {
        if !self.verbosity.is_quiet() {
            println!("{msg}");
        }
    }

    fn error(&mut self, msg: &str) {
        eprintln!("{msg}");
    }

    fn raw_mode(&mut self) {
        if self.raw_mode.is_none() {
            self.raw_mode = stdout().into_raw_mode().ok();
        }
    }

    fn reset(&mut self) {
        if let Some(raw) = &mut self.raw_mode {
            raw.suspend_raw_mode().ok();
        }
    }
}
