use crate::commands::Verbosity;

pub trait Console {
    fn get_verbosity(&mut self) -> Verbosity;
    fn set_verbosity(&mut self, verbosity: Verbosity);
    fn debug(&mut self, msg: &str);
    fn info(&mut self, msg: &str);
    fn error(&mut self, msg: &str);
}
