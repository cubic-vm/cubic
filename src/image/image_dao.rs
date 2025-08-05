use crate::arch::Arch;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageFactory, ImageStore};
use std::path::Path;
use std::str;

pub struct ImageDao {
    fs: FS,
    pub env: Environment,
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
    pub fn new(env: &Environment) -> Result<Self, Error> {
        let fs = FS::new();
        fs.setup_directory_access(&env.get_image_dir())?;
        Result::Ok(ImageDao {
            fs,
            env: env.clone(),
        })
    }
}

impl ImageStore for ImageDao {
    fn get(&self, id: &str) -> Result<Image, Error> {
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

        ImageFactory::new(&self.env)
            .create_images()?
            .iter()
            .find(|image| {
                (image.vendor == vendor)
                    && (image.arch == arch)
                    && (image.codename == name || image.version == name)
            })
            .cloned()
            .ok_or(Error::UnknownImage(id.to_string()))
    }

    fn copy_image(&self, image: &Image, name: &str) -> Result<(), Error> {
        self.fs.create_dir(&self.env.get_instance_dir2(name))?;
        self.fs.copy_file(
            &self.env.get_image_file(&image.to_file_name()),
            &self.env.get_instance_image_file(name),
        )
    }

    fn exists(&self, image: &Image) -> bool {
        Path::new(&format!(
            "{}/{}",
            self.env.get_image_dir(),
            image.to_file_name()
        ))
        .exists()
    }

    fn prune(&self) -> Result<(), Error> {
        self.fs.remove_file(&self.env.get_image_cache_file()).ok();
        self.fs.remove_dir(&self.env.get_image_dir())
    }
}
