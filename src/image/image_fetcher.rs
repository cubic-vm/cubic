use crate::error::{Error, Result};
use crate::models::{HashAlg, Image};
use crate::view::{Console, Spinner, TransferView};
use crate::web::WebClient;
use regex::Regex;
use std::sync::{Arc, LazyLock, Mutex};

static HEX_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9A-Fa-f]+$").unwrap());

pub struct ImageFetcher {}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch_checksum(
        &self,
        client: &mut WebClient,
        image: &Image,
    ) -> Result<Option<(HashAlg, String)>> {
        if let Some(pos) = image.image_url.rfind("/") {
            let file_name = &image.image_url[pos + 1..image.image_url.len()];
            let content = client.download_content(&image.checksum_url)?;
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
                    return Ok(Some((image.hash_alg, hashsum.to_string())));
                }
            }
        }

        Ok(None)
    }

    pub fn fetch(&self, console: &mut dyn Console, image: &Image, target_file: &str) -> Result<()> {
        let mut client = WebClient::new()?;

        let view = Arc::new(Mutex::new(TransferView::new(&format!(
            "Downloading {}",
            &image.to_name()
        ))));
        console.play(view.clone());
        let checksum = client.download_file(&image.image_url, target_file, view)?;
        console.stop();

        // Verify checksum
        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Verify {}",
            image.to_name()
        )))));
        let mut valid_checksum = false;

        if let Ok(Some((hash_alg, hashsum))) = self.fetch_checksum(&mut client, image) {
            let check = match hash_alg {
                HashAlg::Sha512 => checksum.sha512,
                HashAlg::Sha256 => checksum.sha256,
            };
            valid_checksum = check == hashsum;
        }

        console.stop();
        if valid_checksum {
            Ok(())
        } else {
            Err(Error::InvalidChecksum)
        }
    }
}
