use crate::error::Error;
use crate::image::{Image, ImageFetcher, ImageName};
use crate::util::{self, get_home_dir, setup_directory_access};
use regex::Regex;
use std::fs::{create_dir_all, read_dir, remove_file, DirEntry};
use std::path::Path;
use std::str;

pub fn is_image(id: &str) -> bool {
    Regex::new("^[A-Za-z0-9-]+:[A-Za-z0-9-]+$")
        .unwrap()
        .is_match(id)
}

pub struct ImageDao {
    pub image_dir: String,
}

impl ImageDao {
    pub fn new() -> Result<Self, Error> {
        let image_dir = format!("{}/.local/share/cubic/images", get_home_dir()?);
        setup_directory_access(&image_dir)?;

        Result::Ok(ImageDao { image_dir })
    }

    pub fn get_images(&self) -> Vec<ImageName> {
        read_dir(&self.image_dir)
            .map_err(|_| ())
            .and_then(|entries| {
                entries
                    .collect::<Result<Vec<DirEntry>, _>>()
                    .map_err(|_| ())
            })
            .and_then(|entries| {
                entries
                    .iter()
                    .map(|entry| entry.file_name().to_str().map(|x| x.to_string()).ok_or(()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .unwrap_or_default()
            .iter()
            .filter_map(|name: &String| ImageName::from_file_name(name).ok())
            .collect::<Vec<_>>()
    }

    pub fn load(&self, name: &ImageName) -> Result<Image, Error> {
        let path = format!("{}/{}", self.image_dir, name.to_file_name());
        let size = util::get_disk_capacity(&path).unwrap_or(0);
        let name = name.clone();

        Result::Ok(Image { path, name, size })
    }

    pub fn exists(&self, name: &ImageName) -> bool {
        Path::new(&format!("{}/{}", self.image_dir, name.to_file_name())).exists()
    }

    pub fn add(&self, name: &ImageName) -> Result<Image, Error> {
        if !self.exists(name) {
            let fetcher = ImageFetcher::new();
            create_dir_all(&self.image_dir).map_err(Error::Io)?;
            fetcher.fetch(name, &format!("{}/{}", self.image_dir, name.to_file_name()))?;
        }

        self.load(name)
    }

    pub fn delete(&self, name: &ImageName) -> Result<(), Error> {
        remove_file(format!("{}/{}", self.image_dir, name.to_file_name())).map_err(Error::Io)
    }
}
