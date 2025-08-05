use crate::error::Error;
use crate::image::Image;

pub trait ImageStore {
    fn get(&self, id: &str) -> Result<Image, Error>;
    fn copy_image(&self, image: &Image, name: &str) -> Result<(), Error>;
    fn exists(&self, image: &Image) -> bool;
    fn prune(&self) -> Result<(), Error>;
}
