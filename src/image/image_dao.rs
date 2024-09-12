use crate::error::Error;
use crate::image::{Image, ImageFactory, ImageFetcher};
use crate::util;

use std::fs::remove_file;
use std::path::Path;
use std::str;

pub struct ImageDao {
    pub image_dir: String,
    pub images: Vec<Image>,
}

impl ImageDao {
    pub fn new() -> Result<Self, Error> {
        let image_dir = util::get_image_data_dir()?;
        util::setup_directory_access(&image_dir)?;

        Result::Ok(ImageDao {
            image_dir,
            images: ImageFactory::create_images(),
        })
    }

    pub fn get_images(&self) -> &Vec<Image> {
        &self.images
    }

    pub fn get(&self, id: &str) -> Result<Image, Error> {
        let mut tokens = id.split(':');
        let vendor = tokens
            .next()
            .ok_or(Error::InvalidImageName(id.to_string()))?
            .to_string();
        let name = tokens
            .next()
            .ok_or(Error::InvalidImageName(id.to_string()))?
            .to_string();

        self.images
            .iter()
            .find(|image| {
                image.vendor == vendor && (image.codename == name || image.version == name)
            })
            .cloned()
            .ok_or(Error::UnknownImage(id.to_string()))
    }

    pub fn get_disk_capacity(&self, image: &Image) -> Result<u64, Error> {
        let path = format!("{}/{}", self.image_dir, image.to_file_name());
        util::get_disk_capacity(&path)
    }

    pub fn get_disk_size(&self, image: &Image) -> Result<u64, Error> {
        let path = format!("{}/{}", self.image_dir, image.to_file_name());
        util::get_disk_size(&path)
    }

    pub fn copy_image(&self, image: &Image, dir: &str, name: &str) -> Result<(), Error> {
        let path = format!("{}/{}", self.image_dir, image.to_file_name());
        util::create_dir(dir)?;
        util::copy_file(&path, &format!("{dir}/{name}"))
    }

    pub fn exists(&self, image: &Image) -> bool {
        Path::new(&format!("{}/{}", self.image_dir, image.to_file_name())).exists()
    }

    pub fn fetch(&self, image: &Image) -> Result<(), Error> {
        if self.exists(image) {
            Result::Ok(())
        } else {
            util::create_dir(&self.image_dir)?;
            ImageFetcher::new().fetch(
                image,
                &format!("{}/{}", self.image_dir, image.to_file_name()),
            )
        }
    }

    pub fn delete(&self, image: &Image) -> Result<(), Error> {
        remove_file(format!("{}/{}", self.image_dir, image.to_file_name())).map_err(Error::Io)
    }
}
