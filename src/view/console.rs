use crate::commands::Verbosity;
use crate::view::Animation;
use std::sync::{Arc, Mutex};

pub trait Console {
    fn set_verbosity(&mut self, verbosity: Verbosity);
    fn print(&mut self, msg: &str);
    fn debug(&mut self, msg: &str);
    fn info(&mut self, msg: &str);
    fn warn(&mut self, msg: &str);
    fn error(&mut self, msg: &str);

    fn get_geometry(&self) -> Option<(u32, u32)>;

    fn prompt(&mut self, text: &str) -> String;

    fn raw_mode(&mut self);
    fn reset(&mut self);

    fn play(&mut self, animation: Arc<Mutex<dyn Animation>>);
    fn stop(&mut self);
}
