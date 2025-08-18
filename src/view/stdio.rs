use crate::commands::Verbosity;
use crate::view::Console;

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
}
