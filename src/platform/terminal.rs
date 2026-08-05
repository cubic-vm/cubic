use crate::platform::Stream;

pub trait Terminal {
    fn print(&self, stream: Stream, msg: &str);
    fn println(&self, stream: Stream, msg: &str);
    fn flush(&self, stream: Stream);
    fn is_terminal(&self, stream: Stream) -> bool;

    fn read_input(&self) -> String;
    fn read_secret(&self) -> std::result::Result<String, ()>;

    fn raw_mode(&self);
    fn reset(&self);
}
