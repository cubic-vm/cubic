use crate::models::DataSize;
use crate::view::{Animation, ProgressBar};
use std::time::Instant;

pub struct TransferView {
    message: String,
    start_time: Instant,
    transferred_bytes: u64,
    total_bytes: Option<u64>,
    bytes_per_second: u64,
}

impl TransferView {
    pub fn new(message: &str) -> Self {
        TransferView {
            message: message.to_string(),
            start_time: Instant::now(),
            transferred_bytes: 0,
            total_bytes: None,
            bytes_per_second: 0,
        }
    }

    pub fn set_progress(&mut self, transferred_bytes: u64, total_bytes: Option<u64>) {
        self.transferred_bytes = transferred_bytes;
        self.total_bytes = total_bytes;
    }
}

impl Animation for TransferView {
    fn render(&mut self) -> String {
        let mut line = format!(
            "{} {:>10} ",
            self.message,
            DataSize::new(self.transferred_bytes as usize).to_size()
        );

        if let Some(total_bytes) = self.total_bytes {
            let percent = self.transferred_bytes as f64 / total_bytes as f64;
            line += &format!(
                "{} {:>3.0}%",
                ProgressBar::new(percent, 40),
                percent * 100_f64
            );
        }

        let transfer_time_sec = self.start_time.elapsed().as_secs();
        if transfer_time_sec != 0 {
            self.bytes_per_second += self.transferred_bytes / transfer_time_sec;
            self.bytes_per_second /= 2;
            line += &format!(
                " @ {:>8}",
                DataSize::new(self.bytes_per_second as usize).to_speed()
            );
        }

        line
    }
}
