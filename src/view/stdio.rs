use crate::commands::Verbosity;
use crate::view::Console;
use crossterm;

pub struct Stdio {
    verbosity: Verbosity,
}

impl Stdio {
    pub fn new() -> Self {
        Self {
            verbosity: Verbosity::new(false, false),
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
}
