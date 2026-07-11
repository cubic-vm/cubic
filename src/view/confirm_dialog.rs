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

    pub fn confirm(&self, console: &mut dyn Console) -> bool {
        let reply = console.prompt(&format!("{} [y/N]: ", self.message));
        matches!(reply.to_lowercase().as_str(), "y" | "yes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::ConsoleMock;

    #[test]
    fn test_confirm_accepts_y() {
        let console = &mut ConsoleMock::new();
        console.push_input("y");
        assert!(ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_accepts_yes_case_insensitive() {
        let console = &mut ConsoleMock::new();
        console.push_input("Yes");
        assert!(ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_rejects_empty_reply() {
        let console = &mut ConsoleMock::new();
        console.push_input("");
        assert!(!ConfirmDialog::new("Proceed?").confirm(console));
    }

    #[test]
    fn test_confirm_rejects_anything_else() {
        let console = &mut ConsoleMock::new();
        console.push_input("n");
        assert!(!ConfirmDialog::new("Proceed?").confirm(console));
    }
}
