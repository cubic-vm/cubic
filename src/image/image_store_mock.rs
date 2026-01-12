#[cfg(test)]
pub mod tests {

    use crate::error::Error;
    use crate::image::{Image, ImageName, ImageStore};

    #[derive(Default)]
    pub struct ImageStoreMock {
        images: Vec<Image>,
    }

    impl ImageStore for ImageStoreMock {
        fn exists(&self, image: &Image) -> bool {
            self.images.contains(image)
        }

        fn prune(&self) -> Result<(), Error> {
            Ok(())
        }
    }
}
