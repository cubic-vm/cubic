use crate::error::Error;
use crate::image::{HashAlg, Image};
use crate::view::{SpinnerView, TransferView};
use crate::web::WebClient;
use regex::Regex;

pub struct ImageFetcher {}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch_checksum(
        &self,
        client: &mut WebClient,
        image: &Image,
    ) -> Result<Option<(HashAlg, String)>, Error> {
        if let Some(pos) = image.image_url.rfind("/") {
            let regex = Regex::new("^[0-9A-Fa-f]+$").unwrap();
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
                    .filter(|i| regex.is_match(i))
                    .collect::<Vec<_>>();

                if let (&[_], &[hashsum]) = (file_names.as_slice(), hashsums.as_slice()) {
                    return Ok(Some((image.hash_alg, hashsum.to_string())));
                }
            }
        }

        Ok(None)
    }

    pub fn fetch(&self, image: &Image, target_file: &str) -> Result<(), Error> {
        let mut client = WebClient::new()?;

        let checksum = client.download_file(
            &image.image_url,
            target_file,
            TransferView::new(&format!("Downloading {}", &image.to_name())),
        )?;

        // Verify checksum
        let mut spinner = SpinnerView::new(format!("Verify {}", image.to_name()));
        if let Ok(Some((hash_alg, hashsum))) = self.fetch_checksum(&mut client, image) {
            let check = match hash_alg {
                HashAlg::Sha512 => checksum.sha512,
                HashAlg::Sha256 => checksum.sha256,
            };
            spinner.stop();
            if check == hashsum {
                println!("Successfully verified image checksum");
            } else {
                return Err(Error::InvalidChecksum);
            }
        }
        spinner.stop();

        Ok(())
    }
}
