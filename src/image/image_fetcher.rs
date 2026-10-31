use crate::commands::Context;
use crate::error::{Error, Result};
use crate::image::ImageStore;
use crate::models::Image;
use crate::platform::Access;
use crate::util;
use crate::view::TransferView;
use crate::web::{Hasher, WebClient};
use std::io::BufWriter;
use std::sync::Arc;

const WRITE_BUFFER_SIZE: usize = 1 << 20;

#[derive(Default)]
pub struct ImageFetcher;

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher
    }

    async fn fetch_checksum(&self, client: &WebClient, image: &Image) -> Result<Option<String>> {
        let content = client
            .download(&image.get_checksum_url())
            .await?
            .text()
            .await?;
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
                .filter(|i| util::is_hex(i))
                .collect::<Vec<_>>();

            if let (&[_], &[hashsum]) = (file_names.as_slice(), hashsums.as_slice()) {
                return Some(hashsum.to_string());
            }
        }

        let content = content.trim();
        util::is_hex(content).then(|| content.to_string())
    }

    fn find_image_file(listing: &str, glob: &str) -> Option<String> {
        let (prefix, suffix) = glob.split_once('*')?;
        listing
            .split(|c: char| matches!(c, '"' | '\'' | '<' | '>' | '/') || c.is_whitespace())
            .filter(|word| {
                word.strip_prefix(prefix)
                    .is_some_and(|rest| rest.ends_with(suffix))
            })
            .max_by(|a, b| util::compare_natural(a, b))
            .map(str::to_string)
    }

    async fn resolve_image(&self, client: &WebClient, image: &Image) -> Result<Image> {
        if !image.has_image_file_pattern() {
            return Ok(image.clone());
        }

        let listing = client.download(&image.base_url).await?.text().await?;
        let image_file = Self::find_image_file(&listing, &image.image_file)
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

        system.create_dir(&env.get_image_dir())?;
        let target_file = &env.get_image_file(&image.to_file_name());
        let temp_file = &target_file.with_added_extension("tmp");

        let client = WebClient::new()?;
        let image = &self.resolve_image(&client, image).await?;

        let expected = self
            .fetch_checksum(&client, image)
            .await?
            .ok_or(Error::InvalidChecksum)?;

        let download = client.download(&image.get_image_url()).await?;
        let file = system.create_file(temp_file, Access::Normal)?;
        let mut file = BufWriter::with_capacity(WRITE_BUFFER_SIZE, file);
        let message = format!("Downloading {}", &image.to_name());
        let mut progress = TransferView::new(Arc::clone(console), &message, download.size());
        let mut hasher = Hasher::new(image.hash_alg);
        let result = download
            .write_to(&mut [&mut file, &mut progress, &mut hasher])
            .await;
        drop(file);
        let actual = hasher.finalize();
        let result = result.and_then(|()| {
            if actual != expected {
                return Err(Error::InvalidChecksum);
            }
            system.rename_file(temp_file, target_file)
        });
        console.clear_animation();

        if result.is_err() {
            system.remove_file(temp_file).ok();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::{Arch, HashAlg};
    use crate::platform::SystemMock;

    #[tokio::test]
    async fn test_fetch_image_skips_cached_image() {
        let context =
            ContextBuilder::new(SystemMock::new().add_file("images/debian_bookworm_amd64", b""))
                .build();
        let image = Image {
            distro: "debian".to_string(),
            display_name: "Debian".to_string(),
            version: "12".to_string(),
            codename: Some("bookworm".to_string()),
            tags: Vec::new(),
            arch: Arch::AMD64,
            base_url: String::new(),
            image_file: String::new(),
            checksum_file: String::new(),
            hash_alg: HashAlg::Sha512,
            eol: false,
        };

        ImageFetcher::new().fetch(&context, &image).await.unwrap();
    }

    #[test]
    fn test_find_checksum_reads_a_bare_hash() {
        let checksum = ImageFetcher::find_checksum("def456\n", "image.qcow2");
        assert_eq!(checksum, Some("def456".to_string()));
    }

    #[test]
    fn test_find_image_file_picks_the_newest_match() {
        let listing = r#"
<a href="image-20260630.2.qcow2">
<a href="./image-20260630.10.qcow2">
<a href="image-20260630.10.qcow2.sha256">
<a href="image-20260629.12.qcow2">"#;

        let file = ImageFetcher::find_image_file(listing, "image-*.qcow2");
        assert_eq!(file, Some("image-20260630.10.qcow2".to_string()));
        let file = ImageFetcher::find_image_file(listing, "debian-*.qcow2");
        assert_eq!(file, None);
    }
}
