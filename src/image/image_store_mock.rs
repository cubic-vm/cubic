#[cfg(test)]
pub mod tests {
    use crate::arch::Arch;
    use crate::error::Error;
    use crate::image::{Image, ImageStore};

    #[derive(Default)]
    pub struct ImageStoreMock {
        images: Vec<Image>,
    }

    impl ImageStore for ImageStoreMock {
        fn get(&self, id: &str) -> Result<Image, Error> {
            let mut tokens = id.split(':');
            let vendor = tokens
                .next()
                .ok_or(Error::InvalidImageName(id.to_string()))?
                .to_string();
            let name = tokens
                .next()
                .ok_or(Error::InvalidImageName(id.to_string()))?
                .to_string();
            let arch = tokens
                .next()
                .map(Arch::from_str)
                .unwrap_or(Ok(Arch::AMD64))?;

            self.images
                .iter()
                .find(|image| {
                    (image.vendor == vendor)
                        && (image.arch == arch)
                        && (image.codename == name || image.version == name)
                })
                .cloned()
                .ok_or(Error::UnknownInstance(id.to_string()))
        }

        fn copy_image(&self, _image: &Image, _name: &str) -> Result<(), Error> {
            Ok(())
        }

        fn exists(&self, image: &Image) -> bool {
            self.images.contains(image)
        }

        fn prune(&self) -> Result<(), Error> {
            Ok(())
        }
    }
}
