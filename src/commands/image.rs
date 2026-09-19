use crate::commands::Context;
use crate::error::Result;
use crate::image::{ImageFactory, ImageFetcher, ImageStore};
use crate::models::{Image, ImageName};
use crate::view::Spinner;
use std::path::Path;
use std::sync::Arc;

pub async fn fetch_image_list(context: &Context) -> Vec<Image> {
    let console = context.get_console();
    let _spinner = Spinner::new(Arc::clone(console), "Fetching image list".to_string());
    ImageFactory::new(context.get_system(), context.get_env())
        .get_all_images(console)
        .await
        .unwrap_or_default()
}

pub async fn fetch_image_info(context: &Context, image: &ImageName) -> Result<Image> {
    let console = context.get_console();
    let (distro, name) = (image.get_distro(), image.get_name());
    let text = format!("Looking up image {distro}:{name}");
    let _spinner = Spinner::new(Arc::clone(console), text);
    ImageFactory::new(context.get_system(), context.get_env())
        .find_image(console, image)
        .await
}

pub async fn fetch_image(context: &Context, image: &Image) -> Result<()> {
    let (system, env) = (context.get_system(), context.get_env());
    if !ImageStore::new().exists(system, env, image) {
        system.create_writable_dir(Path::new(&env.get_image_dir()))?;
        ImageFetcher::new()
            .fetch(
                context.get_console(),
                system,
                image,
                Path::new(&env.get_image_file(&image.to_file_name())),
            )
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, Environment, HashAlg, UserName};
    use crate::platform::SystemMock;
    use crate::view::Console;
    use std::str::FromStr;
    use std::sync::Arc;

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
            image_url: String::new(),
            checksum_url: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
        };

        // A cached image must return without touching the image directory
        // or the network.
        fetch_image(&context, &image).await.unwrap();
    }
}
