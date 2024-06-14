pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;
pub mod image_name;

pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;
pub use image_name::*;

#[derive(Clone)]
pub struct Image {
    pub vendor: String,
    pub codename: String,
    pub version: String,
    pub url: String,
}

impl Image {
    pub fn to_image_name(&self) -> ImageName {
        ImageName {
            vendor: self.vendor.clone(),
            image: self.codename.clone(),
            arch: "amd64".to_string(),
        }
    }
}
