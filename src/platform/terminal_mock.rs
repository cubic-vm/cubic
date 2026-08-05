use crate::platform::{Stream, SystemMock, Terminal};
use std::collections::VecDeque;

// Both halves of the console, so a test can seed input and read back output
// through a single piece of state.
#[derive(Default)]
pub struct TerminalMock {
    output: String,
    input: VecDeque<String>,
}

impl TerminalMock {
    fn print(&mut self, msg: &str) {
        self.output.push_str(msg);
    }

    fn println(&mut self, msg: &str) {
        self.output.push_str(&format!("{msg}\n"));
    }

    fn get_output(&self) -> String {
        self.output.clone()
    }

    fn push_input(&mut self, line: &str) {
        self.input.push_back(line.to_string());
    }

    // An empty line once the queue is drained, standing in for a console that
    // has nothing left to read.
    fn pop_input(&mut self) -> String {
        self.input.pop_front().unwrap_or_default()
    }
}

impl SystemMock {
    pub fn push_input(&self, line: &str) {
        self.terminal.borrow_mut().push_input(line);
    }

    pub fn get_output(&self) -> String {
        self.terminal.borrow().get_output()
    }
}

impl Terminal for SystemMock {
    fn print(&self, _stream: Stream, msg: &str) {
        self.terminal.borrow_mut().print(msg);
    }

    fn println(&self, _stream: Stream, msg: &str) {
        self.terminal.borrow_mut().println(msg);
    }

    fn flush(&self, _stream: Stream) {}

    // A mocked host is never attended, so colour and prompts stay off.
    fn is_terminal(&self, _stream: Stream) -> bool {
        false
    }

    fn read_input(&self) -> String {
        self.terminal.borrow_mut().pop_input().trim().to_string()
    }

    fn read_secret(&self) -> std::result::Result<String, ()> {
        Ok(self.terminal.borrow_mut().pop_input())
    }

    fn raw_mode(&self) {}

    fn reset(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::platform::{Stream, SystemMock, Terminal};

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
}
