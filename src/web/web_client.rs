use crate::error::Error;
use crate::fs::FS;
use crate::view::TransferView;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

const REQUEST_TIMEOUT_SEC: u64 = 10;

fn hex_encode(bytes: &[u8]) -> String {
    let mut string = String::new();
    for byte in bytes {
        string = format!("{string}{byte:02x}");
    }
    string
}

#[derive(Default)]
pub struct Checksum {
    pub sha512: String,
    pub sha256: String,
}

struct ProgressWriter {
    file: File,
    size: Option<u64>,
    written: u64,
    view: TransferView,
    sha512: Sha512,
    sha256: Sha256,
}

impl ProgressWriter {
    pub fn new(file: File, size: Option<u64>, view: TransferView) -> Self {
        Self {
            file,
            size,
            written: 0,
            view,
            sha512: Sha512::new(),
            sha256: Sha256::new(),
        }
    }
}

impl io::Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written += buf.len() as u64;
        self.sha512.update(buf);
        self.sha256.update(buf);
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
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
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
        &self,
        url: &str,
        file_path: &str,
        view: TransferView,
    ) -> Result<Checksum, Error> {
        let fs = FS::new();

        let temp_file = format!("{file_path}.tmp");
        if Path::new(&temp_file).exists() {
            fs.remove_file(&temp_file)?;
        }

        if Path::new(&file_path).exists() {
            return Result::Ok(Checksum::default());
        }

        let mut resp = reqwest::blocking::get(url).map_err(Error::Web)?;

        let mut writer =
            ProgressWriter::new(fs.create_file(&temp_file)?, resp.content_length(), view);
        resp.copy_to(&mut writer).map_err(Error::Web)?;

        fs.rename_file(&temp_file, file_path)?;

        Ok(Checksum {
            sha512: hex_encode(&writer.sha512.clone().finalize()),
            sha256: hex_encode(&writer.sha256.clone().finalize()),
        })
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
