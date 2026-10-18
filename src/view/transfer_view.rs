use crate::models::DataSize;
use crate::view::{Console, ProgressBar};
use std::io;
use std::sync::Arc;

const TEXT_WIDTH: usize = 50;
const MIN_BAR_WIDTH: usize = 10;

pub struct TransferView {
    console: Arc<Console>,
    message: String,
    transferred_bytes: u64,
    total_bytes: Option<u64>,
}

impl TransferView {
    pub fn new(console: Arc<Console>, message: &str, total_bytes: Option<u64>) -> Self {
        TransferView {
            console,
            message: message.to_string(),
            transferred_bytes: 0,
            total_bytes,
        }
    }

    pub fn with_transferred(mut self, transferred_bytes: u64) -> Self {
        self.transferred_bytes = transferred_bytes;
        self
    }

    pub fn transferred(&self) -> u64 {
        self.transferred_bytes
    }

    fn draw(&self) {
        let frame = self.render(self.console.width(), self.console.has_unicode());
        self.console.update_animation(&frame);
    }

    fn render(&self, width: usize, unicode: bool) -> String {
        let text = format!("{:TEXT_WIDTH$.TEXT_WIDTH$}", self.message);

        let Some(total_bytes) = self.total_bytes else {
            let transferred = DataSize::new(self.transferred_bytes as usize).to_size();
            return format!("{text}{transferred}");
        };

        // Cap at and use the unit of the total to keep the width fixed.
        let transferred_bytes = self.transferred_bytes.min(total_bytes);
        let total = DataSize::new(total_bytes as usize);
        let transferred = DataSize::new(transferred_bytes as usize).to_value_in(&total);
        let size_width = total.to_value_in(&total).to_string().len();
        let percent = transferred_bytes as f64 / total_bytes as f64;
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

impl io::Write for TransferView {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.transferred_bytes += buf.len() as u64;
        self.draw();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    fn new_view(message: &str, transferred_bytes: u64, total_bytes: Option<u64>) -> TransferView {
        TransferView::new(
            Console::new(Arc::new(SystemMock::new())),
            message,
            total_bytes,
        )
        .with_transferred(transferred_bytes)
    }

    #[test]
    fn test_text_column_is_fixed_width() {
        let view = new_view("Downloading ubuntu", 75, Some(100));
        let line = view.render(80, false);
        assert_eq!(line.len(), 80);
        assert!(line.starts_with("Downloading ubuntu"));
        assert_eq!(&line[TEXT_WIDTH..TEXT_WIDTH + 2], " [");
        assert!(line.contains("] "));
    }

    #[test]
    fn test_stats_sit_on_the_right() {
        let view = new_view("Downloading ubuntu", 50, Some(100));
        let line = view.render(80, false);
        assert!(line.ends_with("50%  50/100 B"));
    }

    #[test]
    fn test_stats_stay_put_as_the_size_grows() {
        let total = Some(10 * 1024 * 1024 * 1024);
        let small = new_view("Downloading ubuntu", 1, total).render(80, false);
        let large = new_view("Downloading ubuntu", 5 * 1024 * 1024 * 1024, total).render(80, false);
        assert_eq!(small.find('/'), large.find('/'));
    }

    #[test]
    fn test_long_message_is_truncated() {
        let view = new_view(
            "Downloading a-very-long-image-name-that-overflows-the-column",
            75,
            Some(100),
        );
        let line = view.render(80, false);
        assert_eq!(
            &line[..TEXT_WIDTH],
            "Downloading a-very-long-image-name-that-overflows-"
        );
    }

    #[test]
    fn test_narrow_width_keeps_minimum_bar() {
        let view = new_view("Downloading ubuntu", 50, Some(100));
        let line = view.render(4, false);
        assert!(line.contains('='));
    }

    #[test]
    fn test_no_total_omits_bar() {
        let view = new_view("Downloading ubuntu", 50, None);
        let line = view.render(80, false);
        assert!(!line.contains('['));
    }

    #[test]
    fn test_overshoot_is_capped_at_the_total() {
        let line = new_view("Downloading ubuntu", 150, Some(100)).render(80, false);
        assert_eq!(line.len(), 80);
        assert!(line.ends_with("100% 100/100 B"));
    }

    #[test]
    fn test_write_counts_transferred_bytes() {
        let mut view = new_view("Downloading ubuntu", 0, Some(100));
        io::Write::write_all(&mut view, &[0; 30]).unwrap();
        io::Write::write_all(&mut view, &[0; 20]).unwrap();
        assert!(view.render(80, false).ends_with("50%  50/100 B"));
    }
}
