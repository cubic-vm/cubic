#[cfg(test)]
pub mod tests {

    use crate::error::Error;
    use crate::image::{Image, ImageName, ImageStore};

    #[derive(Default)]
    pub struct ImageStoreMock {
        images: Vec<Image>,
    }

    impl ImageStore for ImageStoreMock {
        fn get(&self, name: &ImageName) -> Result<Image, Error> {
            self.images
                .iter()
                .find(|image| {
                    (image.vendor == name.get_vendor())
                        && (image.arch == name.get_arch())
                        && (image.codename == name.get_name() || image.version == name.get_name())
                })
                .cloned()
                .ok_or(Error::UnknownInstance(name.to_string()))
        }

        fn exists(&self, image: &Image) -> bool {
            self.images.contains(image)
        }

        fn prune(&self) -> Result<(), Error> {
            Ok(())
        }
    }
}
