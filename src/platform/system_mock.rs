#[cfg(test)]
pub mod tests {
    use crate::platform::{Stream, System};
    use std::cell::{Cell, RefCell};
    use std::collections::{HashMap, VecDeque};

    pub struct SystemMock {
        env_vars: HashMap<String, String>,
        output: RefCell<String>,
        terminal: Cell<bool>,
        input: RefCell<VecDeque<String>>,
    }

    impl SystemMock {
        pub fn new() -> Self {
            Self {
                env_vars: HashMap::new(),
                output: RefCell::new(String::new()),
                terminal: Cell::new(false),
                input: RefCell::new(VecDeque::new()),
            }
        }

        pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
            self.env_vars.insert(key.to_string(), value.to_string());
            self
        }

        pub fn set_terminal(self, terminal: bool) -> Self {
            self.terminal.set(terminal);
            self
        }

        pub fn get_output(&self) -> String {
            self.output.borrow().clone()
        }

        pub fn push_input(&self, line: &str) {
            self.input.borrow_mut().push_back(line.to_string());
        }

        fn log(&self, msg: &str) {
            self.output.borrow_mut().push_str(&format!("{msg}\n"));
        }
    }

    impl System for SystemMock {
        fn read_env_var(&self, key: &str) -> Option<String> {
            self.env_vars.get(key).cloned()
        }

        fn print(&self, _stream: Stream, msg: &str) {
            self.output.borrow_mut().push_str(msg);
        }

        fn println(&self, _stream: Stream, msg: &str) {
            self.log(msg);
        }

        fn flush(&self, _stream: Stream) {}

        fn is_terminal(&self, _stream: Stream) -> bool {
            self.terminal.get()
        }

        fn read_input(&self) -> String {
            self.input
                .borrow_mut()
                .pop_front()
                .unwrap_or_default()
                .trim()
                .to_string()
        }

        fn read_secret(&self) -> Result<String, ()> {
            Ok(self.input.borrow_mut().pop_front().unwrap_or_default())
        }

        fn raw_mode(&self) {}

        fn reset(&self) {}
    }

    #[test]
    fn read_env_var_returns_configured_value() {
        let system = SystemMock::new().add_env_var("FOO", "bar");

        assert_eq!(system.read_env_var("FOO"), Some("bar".to_string()));
    }

    #[test]
    fn read_env_var_returns_none_when_not_set() {
        let system = SystemMock::new();

        assert_eq!(system.read_env_var("FOO"), None);
    }

    #[test]
    fn println_appends_message_and_newline_to_output() {
        let system = SystemMock::new();

        system.println(Stream::Stdout, "hello");
        system.println(Stream::Stderr, "world");

        assert_eq!(system.get_output(), "hello\nworld\n");
    }

    #[test]
    fn read_input_returns_queued_input() {
        let system = SystemMock::new();
        system.push_input("first");
        system.push_input("second");

        assert_eq!(system.read_input(), "first");
        assert_eq!(system.read_input(), "second");
        assert_eq!(system.read_input(), "");
    }

    #[test]
    fn is_terminal_defaults_to_false() {
        let system = SystemMock::new();

        assert!(!system.is_terminal(Stream::Stdout));
    }
}
