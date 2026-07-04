use crate::models::DataSize;
use crate::view::{Animation, ProgressBar};

const TEXT_WIDTH: usize = 30;
const MIN_BAR_WIDTH: usize = 10;

pub struct TransferView {
    message: String,
    transferred_bytes: u64,
    total_bytes: Option<u64>,
}

impl TransferView {
    pub fn new(message: &str) -> Self {
        TransferView {
            message: message.to_string(),
            transferred_bytes: 0,
            total_bytes: None,
        }
    }

    pub fn set_progress(&mut self, transferred_bytes: u64, total_bytes: Option<u64>) {
        self.transferred_bytes = transferred_bytes;
        self.total_bytes = total_bytes;
    }
}

impl Animation for TransferView {
    fn render(&mut self, width: usize) -> String {
        let text = format!("{:TEXT_WIDTH$.TEXT_WIDTH$}", self.message);
        let transferred = DataSize::new(self.transferred_bytes as usize).to_size();

        let Some(total_bytes) = self.total_bytes else {
            return format!("{text}{transferred}");
        };

        let total = DataSize::new(total_bytes as usize).to_size();
        let percent = self.transferred_bytes as f64 / total_bytes as f64;
        let stats = format!("{:>3.0}% {transferred} / {total}", percent * 100_f64);
        let bar_width = width
            .saturating_sub(text.len() + 2 + stats.len())
            .max(MIN_BAR_WIDTH);
        format!("{text} {} {stats}", ProgressBar::new(percent, bar_width))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_column_is_fixed_width() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(75, Some(100));
        let line = view.render(80);
        assert_eq!(line.len(), 80);
        assert!(line.starts_with("Downloading ubuntu"));
        assert_eq!(&line[TEXT_WIDTH..TEXT_WIDTH + 2], " [");
        assert!(line.contains("] "));
    }

    #[test]
    fn test_stats_sit_on_the_right() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, Some(100));
        let line = view.render(80);
        assert!(line.ends_with("50% 50   B / 100   B"));
    }

    #[test]
    fn test_long_message_is_truncated_to_thirty() {
        let mut view = TransferView::new("Downloading a-very-long-image-name-that-overflows");
        view.set_progress(75, Some(100));
        let line = view.render(80);
        assert_eq!(&line[..TEXT_WIDTH], "Downloading a-very-long-image-");
    }

    #[test]
    fn test_narrow_width_keeps_minimum_bar() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, Some(100));
        let line = view.render(4);
        assert!(line.contains('='));
    }

    #[test]
    fn test_no_total_omits_bar() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, None);
        let line = view.render(80);
        assert!(!line.contains('['));
    }
}
