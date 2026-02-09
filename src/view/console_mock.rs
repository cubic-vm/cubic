#[cfg(test)]
pub mod tests {

    use crate::commands::Verbosity;
    use crate::view::Console;

    #[derive(Default)]
    pub struct ConsoleMock {
        output: String,
    }

    impl ConsoleMock {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn get_output(&self) -> String {
            self.output.clone()
        }

        fn log(&mut self, msg: &str) {
            self.output = format!("{}{msg}\n", self.output);
        }
    }

    impl Console for ConsoleMock {
        fn set_verbosity(&mut self, _verbosity: Verbosity) {}
        fn get_verbosity(&mut self) -> Verbosity {
            Verbosity::Normal
        }

        fn info(&mut self, msg: &str) {
            self.log(msg)
        }

        fn error(&mut self, msg: &str) {
            self.log(msg)
        }

        fn get_geometry(&self) -> Option<(u32, u32)> {
            None
        }

        fn raw_mode(&mut self) {}
        fn reset(&mut self) {}
    }
}
