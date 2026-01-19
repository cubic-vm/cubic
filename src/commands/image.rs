use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageFactory, ImageFetcher, ImageName, ImageStore};
use crate::view::SpinnerView;

pub fn fetch_image_list(env: &Environment) -> Vec<Image> {
    let mut spinner = SpinnerView::new("Fetching image list".to_string());
    let images: Vec<Image> = ImageFactory::new(env).create_images().unwrap_or_default();
    spinner.stop();
    images
}

pub fn fetch_image_info(env: &Environment, image: &ImageName) -> Result<Image, Error> {
    let mut spinner = SpinnerView::new(format!(
        "Looking up image {}:{}",
        image.get_vendor(),
        image.get_name()
    ));
    let image = ImageFactory::new(env).get_image(image);
    spinner.stop();
    image
}

pub fn fetch_image(
    env: &Environment,
    image_store: &dyn ImageStore,
    image: &Image,
) -> Result<(), Error> {
    if !image_store.exists(image) {
        FS::new().create_dir(&env.get_image_dir())?;
        ImageFetcher::new().fetch(
            image,
            &format!("{}/{}", env.get_image_dir(), image.to_file_name()),
        )?;
    }
    Ok(())
}
