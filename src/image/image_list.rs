use crate::models::Image;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Refreshed before a release with `make generate-image-list`
const IMAGE_LIST: &str = include_str!("images.toml");

static IMAGES: LazyLock<Vec<Image>> = LazyLock::new(|| ImageList::parse(IMAGE_LIST).images);

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct ImageList {
    pub images: Vec<Image>,
}

impl ImageList {
    pub fn read() -> &'static [Image] {
        &IMAGES
    }

    fn parse(content: &str) -> Self {
        toml::from_str(content).expect("The image list that ships with Cubic is broken")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_returns_the_baked_image_list() {
        let images = ImageList::read();

        assert!(!images.is_empty());
        assert!(images.iter().all(|image| {
            !image.distro.is_empty()
                && !image.display_name.is_empty()
                && !image.base_url.is_empty()
                && !image.image_file.is_empty()
                && !image.checksum_file.is_empty()
        }));
    }

    #[test]
    fn test_read_tags_one_stable_and_one_latest_image_per_distro_and_arch() {
        let images = ImageList::read();
        let count_tag = |distro: &str, arch, tag: &str| {
            images
                .iter()
                .filter(|image| image.distro == distro && image.arch == arch)
                .filter(|image| image.tags.iter().any(|t| t == tag))
                .count()
        };

        for image in images {
            assert_eq!(count_tag(&image.distro, image.arch, Image::STABLE_TAG), 1);
            assert_eq!(count_tag(&image.distro, image.arch, Image::LATEST_TAG), 1);
        }
    }
}
