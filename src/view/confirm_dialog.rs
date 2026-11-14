use crate::error::Result;
use crate::view::Console;
use std::sync::Arc;

pub struct ConfirmDialog {
    message: String,
}

impl ConfirmDialog {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn confirm(&self, console: &Arc<Console>) -> Result<bool> {
        let reply = console.prompt(&format!("{} [y/N]: ", self.message), false)?;
        Ok(reply.is_some_and(|r| matches!(r.to_lowercase().as_str(), "y" | "yes")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::platform::SystemMock;

    #[test]
    fn test_confirm_accepts_y() {
        let system = SystemMock::new();
        system.push_input("y");
        let console = &Console::new(Arc::new(system));
        assert!(ConfirmDialog::new("Proceed?").confirm(console).unwrap());
    }

    #[test]
    fn test_confirm_accepts_yes_case_insensitive() {
        let system = SystemMock::new();
        system.push_input("Yes");
        let console = &Console::new(Arc::new(system));
        assert!(ConfirmDialog::new("Proceed?").confirm(console).unwrap());
    }

    #[test]
    fn test_confirm_rejects_empty_reply() {
        let system = SystemMock::new();
        system.push_input("");
        let console = &Console::new(Arc::new(system));
        assert!(!ConfirmDialog::new("Proceed?").confirm(console).unwrap());
    }

    #[test]
    fn test_confirm_rejects_anything_else() {
        let system = SystemMock::new();
        system.push_input("n");
        let console = &Console::new(Arc::new(system));
        assert!(!ConfirmDialog::new("Proceed?").confirm(console).unwrap());
    }

    #[test]
    fn test_confirm_fails_without_a_terminal() {
        let console = &Console::new(Arc::new(SystemMock::new()));
        let result = ConfirmDialog::new("Proceed?").confirm(console);
        assert!(matches!(result, Err(Error::TerminalRequired)));
    }
}
