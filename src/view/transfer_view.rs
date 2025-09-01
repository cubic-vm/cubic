use crate::model::DataSize;
use crate::view::ProgressBar;
use std::io;
use std::io::Write;
use std::time::Instant;

pub struct TransferView {
    message: String,
    start_time: Instant,
    bytes_per_second: u64,
}

impl TransferView {
    pub fn new(message: &str) -> Self {
        TransferView {
            message: message.to_string(),
            start_time: Instant::now(),
            bytes_per_second: 0,
        }
    }

    pub fn update(&mut self, transferred_bytes: u64, total_bytes: Option<u64>) {
        print!(
            "\r{} {:>10} ",
            self.message,
            DataSize::new(transferred_bytes as usize).to_size()
        );

        if let Some(total_bytes) = total_bytes {
            let percent = transferred_bytes as f64 / total_bytes as f64;
            print!(
                "{} {:>3.0}%",
                ProgressBar::new(percent, 40),
                percent * 100_f64
            );
        }

        let transfer_time_sec = self.start_time.elapsed().as_secs();
        if transfer_time_sec != 0 {
            self.bytes_per_second += transferred_bytes / transfer_time_sec;
            self.bytes_per_second /= 2;
            print!(
                " @ {:>8}",
                DataSize::new(self.bytes_per_second as usize).to_speed()
            );
        }

        if total_bytes
            .map(|total| transferred_bytes >= total)
            .unwrap_or(false)
        {
            println!();
        }

        io::stdout().flush().ok();
    }
}
