use crate::error::Result;
use crate::image::Image;

pub trait ImageStore {
    fn exists(&self, image: &Image) -> bool;
    fn prune(&self) -> Result<()>;
}
