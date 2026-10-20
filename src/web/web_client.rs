use crate::error::Result;
use crate::web::Download;
use reqwest::Client;
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const STALL_TIMEOUT: Duration = Duration::from_secs(30);

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
                .build()?,
        })
    }

    pub async fn exists(&self, url: &str) -> Result<bool> {
        Ok(self.client.head(url).send().await?.status().is_success())
    }

    pub async fn download(&self, url: &str) -> Result<Download> {
        let resp = self.client.get(url).send().await?.error_for_status()?;
        Ok(Download { resp })
    }
}
