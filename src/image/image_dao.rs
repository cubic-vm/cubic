use crate::error::Result;
use crate::image::ImageStore;
use crate::models::{Environment, Image};
use crate::platform::System;
use std::path::Path;
use std::rc::Rc;

pub struct ImageDao {
    pub env: Environment,
    system: Rc<dyn System>,
}

impl ImageDao {
    pub fn new(system: Rc<dyn System>, env: &Environment) -> Result<Self> {
        system.create_writable_dir(Path::new(&env.get_image_dir()))?;
        Ok(ImageDao {
            env: env.clone(),
            system,
        })
    }
}

impl ImageStore for ImageDao {
    fn exists(&self, image: &Image) -> bool {
        self.system
            .exists_path(&Path::new(&self.env.get_image_dir()).join(image.to_file_name()))
    }
}
