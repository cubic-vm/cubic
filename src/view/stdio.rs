use crate::view::Console;

pub struct Stdio;

impl Stdio {
    pub fn new() -> Self {
        Stdio {}
    }
}

impl Console for Stdio {
    fn info(&mut self, msg: &str) {
        println!("{msg}");
    }
}
