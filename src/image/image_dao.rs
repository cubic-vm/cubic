use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::{Image, ImageStore};
use std::path::Path;

pub struct ImageDao {
    fs: FS,
    pub env: Environment,
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
