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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
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
    fn test_zero_percent() {
        assert_eq!(&ProgressBar::new(0.0, 12).to_string(), "[>         ]");
    }

    #[test]
    fn test_twenty_five_percent() {
        assert_eq!(&ProgressBar::new(0.25, 12).to_string(), "[==>       ]");
    }

    #[test]
    fn test_fifty_percent() {
        assert_eq!(&ProgressBar::new(0.5, 12).to_string(), "[=====>    ]");
    }

    #[test]
    fn test_seventy_five_percent() {
        assert_eq!(&ProgressBar::new(0.75, 12).to_string(), "[=======>  ]");
    }

    #[test]
    fn test_hundred_percent() {
        assert_eq!(&ProgressBar::new(1.0, 12).to_string(), "[==========]");
    }
}
