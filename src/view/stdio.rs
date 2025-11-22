use crate::commands::Verbosity;
use crate::view::Console;
use libc;
use std::mem;

pub struct Stdio {
    verbosity: Verbosity,
    term: libc::termios,
}

impl Stdio {
    pub fn new() -> Self {
        unsafe {
            let mut term: libc::termios = mem::zeroed();
            libc::tcgetattr(0, &mut term);
            Self {
                verbosity: Verbosity::new(false, false),
                term,
            }
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
        unsafe {
            let mut term = self.term;
            term.c_lflag &= !libc::ICANON;
            term.c_lflag &= !libc::ECHO;
            term.c_lflag &= !libc::ISIG;
            term.c_cc[libc::VMIN] = 1;
            term.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(0, libc::TCSANOW, &term);
        }
    }

    fn reset(&mut self) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.term);
        }
    }
}
