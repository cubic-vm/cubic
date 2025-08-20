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

        fn debug(&mut self, msg: &str) {
            self.log(msg)
        }

        fn info(&mut self, msg: &str) {
            self.log(msg)
        }

        fn error(&mut self, msg: &str) {
            self.log(msg)
        }
    }
}
