use std::cmp::Ordering;
use std::fmt;

pub struct ProgressBar {
    percent: f64,
    size: usize,
    unicode: bool,
}

impl ProgressBar {
    pub fn new(percent: f64, size: usize, unicode: bool) -> Self {
        Self {
            percent,
            size,
            unicode,
        }
    }
}

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        let (open, done, head, rest, close) = if self.unicode {
            (" ", "━", "─", "─", " ")
        } else {
            ("[", "=", ">", " ", "]")
        };
        write!(f, "{open}")?;
        let size = self.size - 2;
        let index = (size as f64 * self.percent) as usize;
        for i in 0..size {
            write!(
                f,
                "{}",
                match i.cmp(&index) {
                    Ordering::Less => done,
                    Ordering::Equal => head,
                    Ordering::Greater => rest,
                }
            )?;
        }
        write!(f, "{close}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_a_bar() {
        let bar = |percent, unicode| ProgressBar::new(percent, 12, unicode).to_string();

        assert_eq!(bar(0.0, false), "[>         ]");
        assert_eq!(bar(0.25, false), "[==>       ]");
        assert_eq!(bar(0.5, false), "[=====>    ]");
        assert_eq!(bar(0.75, false), "[=======>  ]");
        assert_eq!(bar(1.0, false), "[==========]");

        assert_eq!(bar(0.0, true), " ────────── ");
        assert_eq!(bar(0.5, true), " ━━━━━───── ");
        assert_eq!(bar(1.0, true), " ━━━━━━━━━━ ");
    }
}
