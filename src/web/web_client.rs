use crate::error::Error;
use crate::fs::FS;
use crate::view::TransferView;
use reqwest::blocking::Client;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

struct ProgressWriter {
    file: File,
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

pub struct WebClient {
    client: Client,
}

impl WebClient {
    pub fn new() -> Result<Self, Error> {
        Ok(WebClient {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(5))
                .gzip(true)
                .brotli(true)
                .build()
                .map_err(Error::Web)?,
        })
    }

    pub fn get_file_size(&mut self, url: &str) -> Result<Option<u64>, Error> {
        Ok(self
            .client
            .head(url)
            .send()
            .map_err(Error::Web)?
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok()))
    }

    pub fn download_file(
        self,
        url: &str,
        file_path: &str,
        view: TransferView,
    ) -> Result<(), Error> {
        let fs = FS::new();

        let temp_file = format!("{file_path}.tmp");
        if Path::new(&temp_file).exists() {
            fs.remove_file(&temp_file)?;
        }

        if Path::new(&file_path).exists() {
            return Result::Ok(());
        }

        let mut resp = reqwest::blocking::get(url).map_err(Error::Web)?;

        let mut writer = ProgressWriter {
            file: fs.create_file(&temp_file)?,
            size: resp.content_length(),
            written: 0,
            view,
        };
        resp.copy_to(&mut writer).map_err(Error::Web)?;

        fs.rename_file(&temp_file, file_path)
    }

    pub fn download_content(&mut self, url: &str) -> Result<String, Error> {
        self.client
            .get(url)
            .send()
            .map_err(Error::Web)?
            .text()
            .map_err(Error::Web)
    }
}
