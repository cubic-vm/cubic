use crate::image::Image;

pub trait ImageStore {
    fn exists(&self, image: &Image) -> bool;
}
