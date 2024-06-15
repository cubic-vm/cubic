use crate::error::Error;
use crate::image::Image;
use crate::util;
use std::fs;
use std::io;
use std::path::Path;

struct ProgressWriter {
    file: fs::File,
    size: Option<u64>,
    written: u64,
}

impl io::Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(size) = self.size {
            self.written += buf.len() as u64;
            let percent = format!("{: >3.0}%", self.written as f64 / size as f64 * 100_f64);
            print!(
                "\rDownloading image: {:>10} / {:>10} [{percent:>4}]",
                util::bytes_to_human_readable(self.written),
                util::bytes_to_human_readable(size)
            );
            if self.written >= size {
                println!();
            }
        } else {
            print!("\rDownloading image");
        }
        io::stdout().flush().ok();
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

pub struct ImageFetcher {}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch(&self, image: &Image, target_file: &str) -> Result<(), Error> {
        let temp_file = format!("{target_file}.tmp");
        if Path::new(&temp_file).exists() {
            util::remove_file(&temp_file)?;
        }

        if Path::new(&target_file).exists() {
            return Result::Ok(());
        }

        let mut resp = reqwest::blocking::get(&image.url)
            .map_err(|_| Error::ImageDownloadFailed(image.to_id()))?;

        let size = resp.content_length();
        let file = util::create_file(&temp_file)?;
        let mut writer = ProgressWriter {
            file,
            size,
            written: 0,
        };
        resp.copy_to(&mut writer)
            .map_err(|_| Error::ImageDownloadFailed(image.to_id()))?;

        fs::rename(temp_file, target_file).map_err(Error::Io)
    }
}
