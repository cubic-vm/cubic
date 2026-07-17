use crate::view::Console;

pub struct ConfirmDialog {
    message: String,
}

impl ConfirmDialog {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }

    pub fn confirm(&self, console: &mut Console<'_>) -> bool {
        let reply = console.prompt(&format!("{} [y/N]: ", self.message));
        matches!(reply.to_lowercase().as_str(), "y" | "yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_confirm_accepts_y() {
        let system = SystemMock::new();
        system.push_input("y");
        let console = &mut Console::new(&system);
        assert!(ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_accepts_yes_case_insensitive() {
        let system = SystemMock::new();
        system.push_input("Yes");
        let console = &mut Console::new(&system);
        assert!(ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_rejects_empty_reply() {
        let system = SystemMock::new();
        system.push_input("");
        let console = &mut Console::new(&system);
        assert!(!ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_rejects_anything_else() {
        let system = SystemMock::new();
        system.push_input("n");
        let console = &mut Console::new(&system);
        assert!(!ConfirmDialog::new("Proceed?").confirm(console));
    }
}
