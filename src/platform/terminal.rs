use crate::platform::Stream;

pub trait Terminal {
    fn write(&self, stream: Stream, text: &str);
    fn is_terminal(&self, stream: Stream) -> bool;
    fn is_stdin_terminal(&self) -> bool;
    fn get_size(&self) -> Option<(u32, u32)>;

    fn read_input(&self, masked: bool) -> std::result::Result<String, ()>;

    fn set_raw_mode(&self, enabled: bool);
}
