use std::cmp::Ordering;
use std::fmt;

pub struct ProgressBar {
    percent: f64,
    size: usize,
}

impl ProgressBar {
    pub fn new(percent: f64, size: usize) -> Self {
        Self { percent, size }
    }
}

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        write!(f, "[")?;
        let size = self.size - 2;
        let index = (size as f64 * self.percent) as usize;
        for i in 0..size {
            write!(
                f,
                "{}",
                match i.cmp(&index) {
                    Ordering::Less => "=",
                    Ordering::Equal => ">",
                    Ordering::Greater => " ",
                }
            )?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_a_bar() {
        assert_eq!(ProgressBar::new(0.0, 12).to_string(), "[>         ]");
        assert_eq!(ProgressBar::new(0.25, 12).to_string(), "[==>       ]");
        assert_eq!(ProgressBar::new(0.5, 12).to_string(), "[=====>    ]");
        assert_eq!(ProgressBar::new(0.75, 12).to_string(), "[=======>  ]");
        assert_eq!(ProgressBar::new(1.0, 12).to_string(), "[==========]");
    }
}
