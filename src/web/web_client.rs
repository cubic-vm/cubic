use crate::error::Error;
use crate::util;
use crate::view::TransferView;
use std::fs;
use std::io;
use std::path::Path;

struct ProgressWriter {
    file: fs::File,
    size: Option<u64>,
    written: u64,
    view: TransferView,
}

impl io::Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written += buf.len() as u64;
        self.view.update(self.written, self.size);
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

pub struct WebClient {}

impl WebClient {
    pub fn new() -> Self {
        WebClient {}
    }

    pub fn download_file(
        self,
        url: &str,
        file_path: &str,
        view: TransferView,
    ) -> Result<(), Error> {
        let temp_file = format!("{file_path}.tmp");
        if Path::new(&temp_file).exists() {
            util::remove_file(&temp_file)?;
        }

        if Path::new(&file_path).exists() {
            return Result::Ok(());
        }

        let mut resp = reqwest::blocking::get(url).map_err(Error::Web)?;

        let mut writer = ProgressWriter {
            file: util::create_file(&temp_file)?,
            size: resp.content_length(),
            written: 0,
            view,
        };
        resp.copy_to(&mut writer).map_err(Error::Web)?;

        fs::rename(temp_file, file_path).map_err(Error::Io)
    }
}
