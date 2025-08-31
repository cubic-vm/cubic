use crate::error::Error;
use crate::image::{Image, ImageName};

pub trait ImageStore {
    fn get(&self, name: &ImageName) -> Result<Image, Error>;
    fn exists(&self, image: &Image) -> bool;
    fn prune(&self) -> Result<(), Error>;
}
