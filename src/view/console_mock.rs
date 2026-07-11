#[cfg(test)]
pub mod tests {

    use crate::commands::Verbosity;
    use crate::view::{Animation, Console};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    pub struct ConsoleMock {
        output: String,
        input: std::collections::VecDeque<String>,
    }

    impl ConsoleMock {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn get_output(&self) -> String {
            self.output.clone()
        }

        pub fn push_input(&mut self, line: &str) {
            self.input.push_back(line.to_string());
        }

        fn log(&mut self, msg: &str) {
            self.output = format!("{}{msg}\n", self.output);
        }
    }

    impl Console for ConsoleMock {
        fn set_verbosity(&mut self, _verbosity: Verbosity) {}

        fn print(&mut self, msg: &str) {
            self.log(msg)
        }

        fn debug(&mut self, msg: &str) {
            self.log(&format!("debug: {msg}"))
        }

        fn info(&mut self, msg: &str) {
            self.log(&format!("info: {msg}"))
        }

        fn warn(&mut self, msg: &str) {
            self.log(&format!("warn: {msg}"))
        }

        fn error(&mut self, msg: &str) {
            self.log(&format!("error: {msg}"))
        }

        fn get_geometry(&self) -> Option<(u32, u32)> {
            None
        }

        fn prompt(&mut self, text: &str) -> String {
            self.log(text);
            self.input.pop_front().unwrap_or_default()
        }

        fn prompt_password(&mut self, text: &str) -> Result<String, ()> {
            self.log(text);
            Ok(self.input.pop_front().unwrap_or_default())
        }

        fn raw_mode(&mut self) {}
        fn reset(&mut self) {}

        fn play(&mut self, _animation: Arc<Mutex<dyn Animation>>) {}
        fn stop(&mut self) {}
    }
}
