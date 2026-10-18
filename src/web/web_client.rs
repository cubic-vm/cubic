use crate::error::{Error, Result};
use crate::view::{Console, TransferView};
use reqwest::{Client, Response};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const STALL_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ProgressWriter {
    size: Option<u64>,
    written: u64,
    view: TransferView,
    console: Arc<Console>,
}

impl ProgressWriter {
    pub fn new(size: Option<u64>, view: TransferView, console: Arc<Console>) -> Self {
        Self {
            size,
            written: 0,
            view,
            console,
        }
    }
}

impl io::Write for ProgressWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written += buf.len() as u64;
        self.view.set_progress(self.written, self.size);
        self.view.draw(&self.console);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
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

    pub async fn download(&self, url: &str) -> Result<Download> {
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .and_then(Response::error_for_status)
            .map_err(Self::map_error)?;

        Ok(Download { resp })
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

pub struct Download {
    resp: Response,
}

impl Download {
    pub fn size(&self) -> Option<u64> {
        self.resp.content_length()
    }

    pub async fn write_to(mut self, writers: &mut [&mut dyn Write]) -> Result<()> {
        while let Some(chunk) = self.resp.chunk().await.map_err(WebClient::map_error)? {
            for writer in writers.iter_mut() {
                writer.write_all(&chunk).map_err(Error::from)?;
            }
        }

        for writer in writers.iter_mut() {
            writer.flush().map_err(Error::from)?;
        }
        Ok(())
    }
}
