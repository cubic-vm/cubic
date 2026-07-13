#[cfg(test)]
pub mod tests {

    use crate::image::ImageStore;
    use crate::models::Image;

    #[derive(Default)]
    pub struct ImageStoreMock {
        images: Vec<Image>,
    }

    impl ImageStoreMock {
        pub fn new(images: Vec<Image>) -> Self {
            Self { images }
        }
    }

    impl ImageStore for ImageStoreMock {
        fn exists(&self, image: &Image) -> bool {
            self.images.contains(image)
        }
    }
}
