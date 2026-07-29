use crate::error::{Error, Result};
use crate::models::Checksum;
use crate::platform::System;
use crate::util;
use crate::view::TransferView;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256, Sha512};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const REQUEST_TIMEOUT_SEC: u64 = 30;

struct ProgressWriter {
    file: Box<dyn Write>,
    size: Option<u64>,
    written: u64,
    view: Arc<Mutex<TransferView>>,
    sha512: Sha512,
    sha256: Sha256,
}

impl ProgressWriter {
    pub fn new(file: Box<dyn Write>, size: Option<u64>, view: Arc<Mutex<TransferView>>) -> Self {
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
        self.view
            .lock()
            .unwrap()
            .set_progress(self.written, self.size);
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
    pub fn new() -> Result<Self> {
        Ok(WebClient {
            client: reqwest::blocking::Client::builder()
                .user_agent("cubic")
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
                .build()
                .map_err(Error::from)?,
        })
    }

    pub fn get_file_size(&mut self, url: &str) -> Result<Option<u64>> {
        Ok(self
            .client
            .head(url)
            .send()
            .map_err(Error::from)?
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok()))
    }

    pub fn download_file(
        &self,
        system: &dyn System,
        url: &str,
        file_path: &Path,
        view: Arc<Mutex<TransferView>>,
    ) -> Result<Checksum> {
        // Appends rather than replacing the extension, so an image named
        // `foo.img` downloads through `foo.img.tmp`.
        let mut temp_file = file_path.as_os_str().to_owned();
        temp_file.push(".tmp");
        let temp_file = PathBuf::from(temp_file);
        if system.exists_path(&temp_file) {
            system.remove_file(&temp_file)?;
        }

        if system.exists_path(file_path) {
            return Ok(Checksum::default());
        }

        let mut resp = self.client.get(url).send().map_err(Error::from)?;

        let mut writer =
            ProgressWriter::new(system.create_file(&temp_file)?, resp.content_length(), view);
        resp.copy_to(&mut writer).map_err(Error::from)?;

        system.rename_file(&temp_file, file_path)?;

        Ok(Checksum {
            sha512: util::hex_encode(&writer.sha512.clone().finalize()),
            sha256: util::hex_encode(&writer.sha256.clone().finalize()),
        })
    }

    pub fn download_content(&mut self, url: &str) -> Result<String> {
        self.client
            .get(url)
            .send()
            .map_err(Error::from)?
            .text()
            .map_err(Error::from)
    }
}
