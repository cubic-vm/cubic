use crate::error::Error;
use crate::image::{Image, ImageFactory, ImageFetcher, ImageName};
use crate::util;
use regex::Regex;
use std::fs::remove_file;
use std::path::Path;
use std::str;

pub fn is_image(id: &str) -> bool {
    Regex::new("^[A-Za-z0-9-]+:[A-Za-z0-9-]+$")
        .unwrap()
        .is_match(id)
}

pub struct ImageDao {
    pub image_dir: String,
    pub images: Vec<Image>,
}

impl ImageDao {
    pub fn new() -> Result<Self, Error> {
        let image_dir = format!("{}/.local/share/cubic/images", util::get_home_dir()?);
        util::setup_directory_access(&image_dir)?;

        Result::Ok(ImageDao {
            image_dir,
            images: ImageFactory::create_images(),
        })
    }

    pub fn get_images(&self) -> &Vec<Image> {
        &self.images
    }

    pub fn load(&self, name: &ImageName) -> Result<Image, Error> {
        self.images
            .iter()
            .find(|image| image.vendor == name.vendor && image.codename == name.image)
            .cloned()
            .ok_or(Error::UnknownImage(name.clone()))
    }

    pub fn get_capacity(&self, image: &Image) -> Result<u64, Error> {
        let path = format!(
            "{}/{}",
            self.image_dir,
            image.to_image_name().to_file_name()
        );
        util::get_disk_capacity(&path)
    }

    pub fn copy_image(&self, image: &Image, dir: &str, name: &str) -> Result<(), Error> {
        let path = format!(
            "{}/{}",
            self.image_dir,
            image.to_image_name().to_file_name()
        );
        util::create_dir(dir)?;
        util::copy_file(&path, &format!("{dir}/{name}"))
    }

    pub fn exists(&self, name: &ImageName) -> bool {
        Path::new(&format!("{}/{}", self.image_dir, name.to_file_name())).exists()
    }

    pub fn add(&self, name: &ImageName) -> Result<Image, Error> {
        if !self.exists(name) {
            let image = self.load(name)?;
            let fetcher = ImageFetcher::new();
            util::create_dir(&self.image_dir)?;
            fetcher.fetch(
                &image,
                &format!("{}/{}", self.image_dir, name.to_file_name()),
            )?;
        }

        self.load(name)
    }

    pub fn delete(&self, name: &ImageName) -> Result<(), Error> {
        remove_file(format!("{}/{}", self.image_dir, name.to_file_name())).map_err(Error::Io)
    }
}
