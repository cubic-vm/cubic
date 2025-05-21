pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;

use crate::arch::Arch;
pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;

#[derive(Clone)]
pub struct Image {
    pub vendor: String,
    pub codename: String,
    pub version: String,
    pub arch: Arch,
    pub url: String,
    pub size: Option<u64>,
}

impl Image {
    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.vendor, self.codename, self.arch)
    }

    pub fn to_id(&self) -> String {
        format!("{}:{}", self.vendor, self.codename)
    }
}
