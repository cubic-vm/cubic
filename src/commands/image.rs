use crate::error::Result;
use crate::image::{ImageFactory, ImageFetcher, ImageStore};
use crate::models::{Environment, Image, ImageName};
use crate::platform::System;
use crate::view::{Console, Spinner};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub fn fetch_image_list(
    console: &mut Console<'_>,
    system: &dyn System,
    env: &Environment,
) -> Vec<Image> {
    console.play(Arc::new(Mutex::new(Spinner::new(
        "Fetching image list".to_string(),
    ))));
    let images: Vec<Image> = ImageFactory::new(system, env)
        .get_all_images(console)
        .unwrap_or_default();
    console.stop();
    images
}

pub fn fetch_image_info(
    console: &mut Console<'_>,
    system: &dyn System,
    env: &Environment,
    image: &ImageName,
) -> Result<Image> {
    console.play(Arc::new(Mutex::new(Spinner::new(format!(
        "Looking up image {}:{}",
        image.get_vendor(),
        image.get_name()
    )))));
    let image = ImageFactory::new(system, env).find_image(console, image);
    console.stop();
    image
}

pub fn fetch_image(
    console: &mut Console<'_>,
    system: &dyn System,
    env: &Environment,
    image: &Image,
) -> Result<()> {
    if !ImageStore::new().exists(system, env, image) {
        system.create_writable_dir(Path::new(&env.get_image_dir()))?;
        ImageFetcher::new().fetch(
            console,
            system,
            image,
            Path::new(&env.get_image_file(&image.to_file_name())),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Arch, HashAlg, UserName};
    use crate::platform::SystemMock;
    use std::str::FromStr;

    #[test]
    fn test_fetch_image_skips_cached_image() {
        let system = SystemMock::new().add_file("images/debian_bookworm_amd64", b"");
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
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

        // A cached image must return without touching the image directory
        // or the network.
        fetch_image(console, &system, &env, &image).unwrap();
    }
}
