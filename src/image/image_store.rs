use crate::models::Image;

pub trait ImageStore {
    fn exists(&self, image: &Image) -> bool;
}
