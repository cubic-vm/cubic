pub mod image_dao;
pub mod image_fetcher;
pub mod image_name;

pub use image_dao::*;
pub use image_fetcher::*;
pub use image_name::*;

#[derive(Clone)]
pub struct Image {
    pub path: String,
    pub name: ImageName,
    pub size: u64,
}
