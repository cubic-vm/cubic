use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};

#[derive(Clone, Copy)]
pub struct Styler {
    enabled: bool,
}

impl Styler {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn bold(&self, text: &str) -> String {
        if self.enabled {
            let bold = SetAttribute(Attribute::Bold);
            let reset = SetAttribute(Attribute::Reset);
            format!("{bold}{text}{reset}")
        } else {
            text.to_string()
        }
    }

    pub fn bold_color(&self, text: &str, color: Color) -> String {
        if self.enabled {
            format!("{}{}", SetForegroundColor(color), self.bold(text))
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styler_leaves_text_unchanged_when_disabled() {
        let styler = Styler::new(false);
        assert_eq!(styler.bold("Name"), "Name");
        assert_eq!(styler.bold_color("error:", Color::Red), "error:");
    }
}
