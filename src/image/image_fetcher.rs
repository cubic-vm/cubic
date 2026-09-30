use crate::commands::Context;
use crate::error::{Error, Result};
use crate::image::ImageStore;
use crate::models::Image;
use crate::util;
use crate::view::{Spinner, TransferView};
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
        let content = client.download_content(&image.get_checksum_url()).await?;
        Ok(Self::find_checksum(&content, &image.image_file))
    }

    fn find_checksum(content: &str, file_name: &str) -> Option<String> {
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
                return Some(hashsum.to_string());
            }
        }

        let content = content.trim();
        HEX_REGEX.is_match(content).then(|| content.to_string())
    }

    async fn resolve_image(&self, client: &mut WebClient, image: &Image) -> Result<Image> {
        if !image.has_image_file_pattern() {
            return Ok(image.clone());
        }

        let listing = client.download_content(&image.base_url).await?;
        let pattern = util::convert_glob_to_regex(&image.image_file);
        let image_file = util::find_newest_file(&pattern, &listing)
            .ok_or_else(|| Error::NoImageFound(image.base_url.clone()))?;

        Ok(Image {
            image_file,
            ..image.clone()
        })
    }

    pub async fn fetch(&self, context: &Context, image: &Image) -> Result<()> {
        let (console, system, env) = (
            context.get_console(),
            context.get_system(),
            context.get_env(),
        );
        if ImageStore::new().exists(system, env, image) {
            return Ok(());
        }

        system.create_writable_dir(Path::new(&env.get_image_dir()))?;
        let target_file = &env.get_image_file(&image.to_file_name());
        let target_file = Path::new(target_file);

        let mut client = WebClient::new()?;
        let image = &self.resolve_image(&mut client, image).await?;

        let view = TransferView::new(&format!("Downloading {}", &image.to_name()));
        let checksum = client
            .download_file(
                system,
                &image.get_image_url(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, Environment, HashAlg, UserName};
    use crate::platform::SystemMock;
    use crate::view::Console;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_fetch_image_skips_cached_image() {
        let system = Arc::new(SystemMock::new().add_file("images/debian_bookworm_amd64", b""));
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        let context = Context::new(
            Arc::clone(&system) as Arc<dyn crate::platform::System>,
            Console::new(Arc::new(SystemMock::new())),
            env,
            Box::new(InstanceStoreMock::new(Vec::new())),
        );
        let image = Image {
            distro: "debian".to_string(),
            version: "12".to_string(),
            codename: Some("bookworm".to_string()),
            tags: Vec::new(),
            arch: Arch::AMD64,
            base_url: String::new(),
            image_file: String::new(),
            checksum_file: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
            eol: false,
        };

        ImageFetcher::new().fetch(&context, &image).await.unwrap();
    }

    #[test]
    fn test_find_checksum_reads_a_bare_hash() {
        assert_eq!(
            ImageFetcher::find_checksum("def456\n", "image.qcow2"),
            Some("def456".to_string())
        );
    }
}
