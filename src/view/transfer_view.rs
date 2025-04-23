use crate::util;
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

    pub fn update(&mut self, transfered_bytes: u64, total_bytes: Option<u64>) {
        print!(
            "\r{}: {:>10}",
            self.message,
            util::bytes_to_human_readable(transfered_bytes)
        );

        if let Some(total_bytes) = total_bytes {
            print!(
                " / {:>10} [{:>3.0}%]",
                util::bytes_to_human_readable(total_bytes),
                transfered_bytes as f64 / total_bytes as f64 * 100_f64
            );
        }

        let transfer_time_sec = self.start_time.elapsed().as_secs();
        if transfer_time_sec != 0 {
            self.bytes_per_second += transfered_bytes / transfer_time_sec;
            self.bytes_per_second /= 2;
            print!(
                " {:>10}/s",
                util::bytes_to_human_readable(self.bytes_per_second)
            );
        }

        if total_bytes
            .map(|total| transfered_bytes >= total)
            .unwrap_or(false)
        {
            println!();
        }

        io::stdout().flush().ok();
    }
}
