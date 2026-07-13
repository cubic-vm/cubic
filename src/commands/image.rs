use crate::error::Result;
use crate::fs::FS;
use crate::image::{ImageFactory, ImageFetcher, ImageStore};
use crate::models::{Environment, Image, ImageName};
use crate::view::{Console, Spinner};
use std::sync::{Arc, Mutex};

pub fn fetch_image_list(console: &mut dyn Console, env: &Environment) -> Vec<Image> {
    console.play(Arc::new(Mutex::new(Spinner::new(
        "Fetching image list".to_string(),
    ))));
    let images: Vec<Image> = ImageFactory::new(env)
        .get_all_images(console)
        .unwrap_or_default();
    console.stop();
    images
}

pub fn fetch_image_info(
    console: &mut dyn Console,
    env: &Environment,
    image: &ImageName,
) -> Result<Image> {
    console.play(Arc::new(Mutex::new(Spinner::new(format!(
        "Looking up image {}:{}",
        image.get_vendor(),
        image.get_name()
    )))));
    let image = ImageFactory::new(env).find_image(console, image);
    console.stop();
    image
}

pub fn fetch_image(
    console: &mut dyn Console,
    env: &Environment,
    image_store: &dyn ImageStore,
    image: &Image,
) -> Result<()> {
    if !image_store.exists(image) {
        FS::new().create_dir(&env.get_image_dir())?;
        ImageFetcher::new().fetch(
            console,
            image,
            &format!("{}/{}", env.get_image_dir(), image.to_file_name()),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::models::{Arch, HashAlg};
    use crate::view::ConsoleMock;

    #[test]
    fn test_fetch_image_skips_cached_image() {
        let console = &mut ConsoleMock::new();
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let image = Image {
            vendor: "debian".to_string(),
            names: vec!["12".to_string(), "bookworm".to_string()],
            arch: Arch::AMD64,
            image_url: String::new(),
            checksum_url: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
        };
        let image_store = ImageStoreMock::new(vec![image.clone()]);

        // A cached image must return without touching the image directory
        // or the network.
        fetch_image(console, &env, &image_store, &image).unwrap();
    }
}
