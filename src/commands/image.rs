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
    let images: Vec<Image> = ImageFactory::new(env).get_all_images().unwrap_or_default();
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
    let image = ImageFactory::new(env).find_image(image);
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
