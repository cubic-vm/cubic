use crate::error::{Error, Result};
use crate::models::HashAlg;
use crate::platform::System;
use crate::view::{Console, TransferView};
use crate::web::Hasher;
use reqwest::{Client, Response};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const STALL_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_BUFFER_SIZE: usize = 1 << 20;

struct ProgressWriter {
    file: BufWriter<Box<dyn Write>>,
    size: Option<u64>,
    written: u64,
    view: TransferView,
    console: Arc<Console>,
    hasher: Hasher,
}

impl ProgressWriter {
    pub fn new(
        file: Box<dyn Write>,
        size: Option<u64>,
        view: TransferView,
        console: Arc<Console>,
        hash_alg: HashAlg,
    ) -> Self {
        Self {
            file: BufWriter::with_capacity(WRITE_BUFFER_SIZE, file),
            size,
            written: 0,
            view,
            console,
            hasher: Hasher::new(hash_alg),
        }
    }
}

impl io::Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written += buf.len() as u64;
        self.hasher.update(buf);
        self.view.set_progress(self.written, self.size);
        self.console
            .update_animation(&self.view.render(self.console.width()));
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
            client: Client::builder()
                .user_agent("cubic")
                .connect_timeout(CONNECT_TIMEOUT)
                .read_timeout(STALL_TIMEOUT)
                .build()
                .map_err(Self::map_error)?,
        })
    }

    fn map_error(error: reqwest::Error) -> Error {
        if error.is_timeout() {
            Error::WebTimeout
        } else {
            Error::Web(error)
        }
    }

    pub async fn get_file_size(&mut self, url: &str) -> Result<Option<u64>> {
        Ok(self
            .client
            .head(url)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(Self::map_error)?
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok()))
    }

    pub async fn download_file(
        &self,
        system: &dyn System,
        url: &str,
        file_path: &Path,
        view: TransferView,
        console: Arc<Console>,
        hash_alg: HashAlg,
    ) -> Result<String> {
        if system.exists_path(file_path) {
            return Ok(String::new());
        }

        // Appends rather than replacing the extension, so an image named
        // `foo.img` downloads through `foo.img.tmp`.
        let mut temp_file = file_path.as_os_str().to_owned();
        temp_file.push(".tmp");
        let temp_file = PathBuf::from(temp_file);
        system.remove_file(&temp_file).ok();

        let mut resp = self.client.get(url).send().await.map_err(Self::map_error)?;

        let mut writer = ProgressWriter::new(
            system.create_file(&temp_file)?,
            resp.content_length(),
            view,
            console,
            hash_alg,
        );
        if let Err(error) = Self::write_body(&mut resp, &mut writer).await {
            system.remove_file(&temp_file).ok();
            return Err(error);
        }

        system.rename_file(&temp_file, file_path)?;

        Ok(writer.hasher.finalize())
    }

    async fn write_body(resp: &mut Response, writer: &mut ProgressWriter) -> Result<()> {
        while let Some(chunk) = resp.chunk().await.map_err(Self::map_error)? {
            writer.write_all(&chunk).map_err(Error::from)?;
        }

        // The buffered writer drops its tail without this flush
        writer.flush().map_err(Error::from)
    }

    pub async fn download_content(&mut self, url: &str) -> Result<String> {
        self.client
            .get(url)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(Self::map_error)?
            .text()
            .await
            .map_err(Self::map_error)
    }
}
