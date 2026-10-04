use crate::models::DataSize;
use crate::view::{Console, ProgressBar};

const TEXT_WIDTH: usize = 50;
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

    pub fn draw(&self, console: &Console) {
        console.update_animation(&self.render(console.width(), console.has_unicode()));
    }

    fn render(&self, width: usize, unicode: bool) -> String {
        let text = format!("{:TEXT_WIDTH$.TEXT_WIDTH$}", self.message);

        let Some(total_bytes) = self.total_bytes else {
            let transferred = DataSize::new(self.transferred_bytes as usize).to_size();
            return format!("{text}{transferred}");
        };

        // Show the transferred size in the unit of the total so the two read
        // as a fraction. Its value never has more digits than the total, so
        // the field width stays fixed and nothing shifts during a transfer.
        let total = DataSize::new(total_bytes as usize);
        let transferred = DataSize::new(self.transferred_bytes as usize).to_value_in(&total);
        let size_width = total.to_value_in(&total).to_string().len();
        let percent = self.transferred_bytes as f64 / total_bytes as f64;
        let stats = format!(
            "{:>3.0}% {transferred:>size_width$}/{}",
            percent * 100_f64,
            total.to_size()
        );
        let bar_width = width
            .saturating_sub(TEXT_WIDTH + 2 + stats.len())
            .max(MIN_BAR_WIDTH);
        format!(
            "{text} {} {stats}",
            ProgressBar::new(percent, bar_width, unicode)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_column_is_fixed_width() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(75, Some(100));
        let line = view.render(80, false);
        assert_eq!(line.len(), 80);
        assert!(line.starts_with("Downloading ubuntu"));
        assert_eq!(&line[TEXT_WIDTH..TEXT_WIDTH + 2], " [");
        assert!(line.contains("] "));
    }

    #[test]
    fn test_stats_sit_on_the_right() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, Some(100));
        let line = view.render(80, false);
        assert!(line.ends_with("50%  50/100 B"));
    }

    #[test]
    fn test_stats_stay_put_as_the_size_grows() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(1, Some(10 * 1024 * 1024 * 1024));
        let small = view.render(80, false);
        view.set_progress(5 * 1024 * 1024 * 1024, Some(10 * 1024 * 1024 * 1024));
        let large = view.render(80, false);
        assert_eq!(small.find('/'), large.find('/'));
    }

    #[test]
    fn test_long_message_is_truncated() {
        let mut view =
            TransferView::new("Downloading a-very-long-image-name-that-overflows-the-column");
        view.set_progress(75, Some(100));
        let line = view.render(80, false);
        assert_eq!(
            &line[..TEXT_WIDTH],
            "Downloading a-very-long-image-name-that-overflows-"
        );
    }

    #[test]
    fn test_narrow_width_keeps_minimum_bar() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, Some(100));
        let line = view.render(4, false);
        assert!(line.contains('='));
    }

    #[test]
    fn test_no_total_omits_bar() {
        let mut view = TransferView::new("Downloading ubuntu");
        view.set_progress(50, None);
        let line = view.render(80, false);
        assert!(!line.contains('['));
    }
}
