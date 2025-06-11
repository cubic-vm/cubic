use crate::arch::Arch;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageFactory};
use crate::util;
use std::path::Path;
use std::str;

pub struct ImageDao {
    fs: FS,
    pub image_dir: String,
}

#[cfg(target_arch = "aarch64")]
fn get_default_arch() -> Arch {
    Arch::ARM64
}

#[cfg(not(target_arch = "aarch64"))]
fn get_default_arch() -> Arch {
    Arch::AMD64
}

impl ImageDao {
    pub fn new() -> Result<Self, Error> {
        let fs = FS::new();
        let image_dir = util::get_image_data_dir()?;
        fs.setup_directory_access(&image_dir)?;

        Result::Ok(ImageDao { fs, image_dir })
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
        let arch = tokens
            .next()
            .map(Arch::from_str)
            .unwrap_or(Ok(get_default_arch()))?;

        ImageFactory::create_images_for_distro(&vendor)?
            .iter()
            .find(|image| (image.arch == arch) && (image.codename == name || image.version == name))
            .cloned()
            .ok_or(Error::UnknownImage(id.to_string()))
    }

    pub fn get_disk_capacity(&self, image: &Image) -> Result<u64, Error> {
        let path = format!("{}/{}", self.image_dir, image.to_file_name());
        util::get_disk_capacity(&path)
    }

    pub fn copy_image(&self, image: &Image, dir: &str, name: &str) -> Result<(), Error> {
        let path = format!("{}/{}", self.image_dir, image.to_file_name());
        self.fs.create_dir(dir)?;
        self.fs.copy_file(&path, &format!("{dir}/{name}"))
    }

    pub fn exists(&self, image: &Image) -> bool {
        Path::new(&format!("{}/{}", self.image_dir, image.to_file_name())).exists()
    }

    pub fn delete(&self, image: &Image) -> Result<(), Error> {
        self.fs
            .remove_file(&format!("{}/{}", self.image_dir, image.to_file_name()))
    }

    pub fn prune(&self) -> Result<(), Error> {
        self.fs.remove_dir(&self.image_dir)
    }
}
