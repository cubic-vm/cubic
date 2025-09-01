use crate::error::Error;
use crate::image::Image;
use crate::view::TransferView;
use crate::web::WebClient;

pub struct ImageFetcher {}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch(&self, image: &Image, target_file: &str) -> Result<(), Error> {
        WebClient::new()?.download_file(
            &image.url,
            target_file,
            TransferView::new(&format!("Downloading {}", &image.to_name())),
        )
    }
}
