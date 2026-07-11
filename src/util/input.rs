use crate::view::Console;

pub fn confirm(console: &mut dyn Console, text: &str) -> bool {
    console.prompt(text) == "y"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::ConsoleMock;

    #[test]
    fn test_confirm_accepts_y() {
        let console = &mut ConsoleMock::new();
        console.push_input("y");
        assert!(confirm(console, "Proceed? [y/n]: "));
    }

    #[test]
    fn test_confirm_rejects_anything_else() {
        let console = &mut ConsoleMock::new();
        console.push_input("n");
        assert!(!confirm(console, "Proceed? [y/n]: "));
    }
}
