pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;

pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;

#[derive(Clone)]
pub struct Image {
    pub vendor: String,
    pub codename: String,
    pub version: String,
    pub url: String,
    pub size: Option<u64>,
}

impl Image {
    pub fn to_file_name(&self) -> String {
        format!("{}_{}_amd64", self.vendor, self.codename)
    }

    pub fn to_id(&self) -> String {
        format!("{}:{}", self.vendor, self.codename)
    }
}
