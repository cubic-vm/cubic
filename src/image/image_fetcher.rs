use crate::error::{Error, Result};
use crate::models::Image;
use crate::platform::System;
use crate::view::{Console, Spinner, TransferView};
use crate::web::WebClient;
use regex::Regex;
use std::path::Path;
use std::sync::{Arc, LazyLock};

static HEX_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9A-Fa-f]+$").unwrap());

#[derive(Default)]
pub struct ImageFetcher;

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher
    }

    pub async fn fetch_checksum(
        &self,
        client: &mut WebClient,
        image: &Image,
    ) -> Result<Option<String>> {
        if let Some(pos) = image.image_url.rfind("/") {
            let file_name = &image.image_url[pos + 1..image.image_url.len()];
            let content = client.download_content(&image.checksum_url).await?;
            for line in content.lines() {
                let line = line
                    .replace("*", "")
                    .replace("=", "")
                    .replace("(", " ")
                    .replace(")", "")
                    .replace("  ", " ");
                let tokens = line.split(" ").collect::<Vec<_>>();

                let file_names = tokens
                    .iter()
                    .filter(|i| *i == &file_name)
                    .collect::<Vec<_>>();
                let hashsums = tokens
                    .iter()
                    .filter(|i| HEX_REGEX.is_match(i))
                    .collect::<Vec<_>>();

                if let (&[_], &[hashsum]) = (file_names.as_slice(), hashsums.as_slice()) {
                    return Ok(Some(hashsum.to_string()));
                }
            }
        }

        Ok(None)
    }

    pub async fn fetch(
        &self,
        console: &Arc<Console>,
        system: &dyn System,
        image: &Image,
        target_file: &Path,
    ) -> Result<()> {
        let mut client = WebClient::new()?;

        let view = TransferView::new(&format!("Downloading {}", &image.to_name()));
        let checksum = client
            .download_file(
                system,
                &image.image_url,
                target_file,
                view,
                Arc::clone(console),
                image.hash_alg,
            )
            .await?;
        console.clear_animation();

        // Verify checksum
        let mut valid_checksum = false;
        {
            let _spinner = Spinner::new(Arc::clone(console), format!("Verify {}", image.to_name()));
            if let Ok(Some(hashsum)) = self.fetch_checksum(&mut client, image).await {
                valid_checksum = checksum == hashsum;
            }
        }

        if valid_checksum {
            Ok(())
        } else {
            system.remove_file(target_file).ok();
            Err(Error::InvalidChecksum)
        }
    }
}
