use crate::error::{Error, Result};
use crate::models::HashAlg;
use crate::view::{Console, TransferView};
use crate::web::Hasher;
use reqwest::{Client, Response};
use std::io::{self, BufWriter, Write};
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
        self.view.draw(&self.console);
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[derive(Clone)]
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

    pub async fn check_file_exists(&self, url: &str) -> Result<bool> {
        Ok(self
            .client
            .head(url)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(Self::map_error)?
            .status()
            .is_success())
    }

    pub async fn download_file(
        &self,
        url: &str,
        file: Box<dyn Write>,
        view: TransferView,
        console: Arc<Console>,
        hash_alg: HashAlg,
    ) -> Result<String> {
        let mut resp = self
            .client
            .get(url)
            .send()
            .await
            .and_then(Response::error_for_status)
            .map_err(Self::map_error)?;

        let mut writer = ProgressWriter::new(file, resp.content_length(), view, console, hash_alg);
        Self::write_body(&mut resp, &mut writer).await?;

        Ok(writer.hasher.finalize())
    }

    async fn write_body(resp: &mut Response, writer: &mut ProgressWriter) -> Result<()> {
        while let Some(chunk) = resp.chunk().await.map_err(Self::map_error)? {
            writer.write_all(&chunk).map_err(Error::from)?;
        }

        // The buffered writer drops its tail without this flush
        writer.flush().map_err(Error::from)
    }

    pub async fn download_content(&self, url: &str) -> Result<String> {
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
