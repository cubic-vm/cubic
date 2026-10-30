use crate::platform::{Stream, SystemMock, Terminal};
use std::collections::VecDeque;

// Both halves of the console, so a test can seed input and read back output
// through a single piece of state.
#[derive(Default)]
pub struct TerminalMock {
    output: String,
    input: VecDeque<String>,
    is_terminal: bool,
}

impl TerminalMock {
    fn write(&mut self, text: &str) {
        self.output.push_str(text);
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
    pub fn set_terminal(self, is_terminal: bool) -> Self {
        self.terminal.lock().unwrap().is_terminal = is_terminal;
        self
    }

    pub fn push_input(&self, line: &str) {
        self.terminal.lock().unwrap().push_input(line);
    }

    pub fn get_output(&self) -> String {
        self.terminal.lock().unwrap().get_output()
    }
}

impl Terminal for SystemMock {
    fn write(&self, _stream: Stream, text: &str) {
        self.terminal.lock().unwrap().write(text);
    }

    // A mocked host is unattended by default, so colour and animations stay
    // off until a test asks for them.
    fn is_terminal(&self, _stream: Stream) -> bool {
        self.terminal.lock().unwrap().is_terminal
    }

    fn is_stdin_terminal(&self) -> bool {
        false
    }

    fn get_size(&self) -> Option<(u32, u32)> {
        Some((80, 24))
    }

    fn read_input(&self, _masked: bool) -> std::result::Result<String, ()> {
        Ok(self.terminal.lock().unwrap().pop_input())
    }

    fn raw_mode(&self) {}

    fn reset(&self) {}
}
