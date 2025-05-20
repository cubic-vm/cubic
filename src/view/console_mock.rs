#[cfg(test)]
pub mod tests {

    use crate::view::Console;

    #[derive(Default)]
    pub struct ConsoleMock {
        output: String,
    }

    impl ConsoleMock {
        pub fn new() -> Self {
            ConsoleMock::default()
        }

        pub fn get_output(&self) -> String {
            self.output.clone()
        }
    }

    impl Console for ConsoleMock {
        fn info(&mut self, msg: &str) {
            self.output = format!("{}{msg}\n", self.output);
        }
    }
}
