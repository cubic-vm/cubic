use crate::error::{Error, Result};
use crate::image::{self, ImageList};
use crate::models::{Arch, Image, ImageName};
use crate::util;
use crate::view::Console;
use crate::web::WebClient;
use std::sync::Arc;

const IMAGE_PROVIDERS: &[&dyn image::ImageProvider] = &[
    &image::AlmaLinuxImageProvider {},
    &image::AlpineImageProvider {},
    &image::ArchLinuxImageProvider {},
    &image::CentOsImageProvider {},
    &image::DebianImageProvider {},
    &image::FedoraImageProvider {},
    &image::GentooImageProvider {},
    &image::OpenSuseLeapImageProvider {},
    &image::OpenSuseTumbleweedImageProvider {},
    &image::RockyLinuxImageProvider {},
    &image::UbuntuImageProvider {},
];

pub struct ImageFactory;

impl ImageFactory {
    async fn get_images_from_provider_name_arch(
        console: &Arc<Console>,
        web: &WebClient,
        image_provider: &dyn image::ImageProvider,
        name: &str,
        arch: Arch,
    ) -> Option<Image> {
        let image_dir_path = image_provider.get_image_dir_path(name, arch);
        let image_dir_url = format!("{}{image_dir_path}", image_provider.get_base_url());
        // The content URL serves the image and its checksum from one host
        let content_dir_url = format!("{}{image_dir_path}", image_provider.get_content_url());
        console.debug(&format!(
            "Fetching image directory listing '{image_dir_url}'"
        ));
        let image_content = match web.download_content(&image_dir_url).await {
            Ok(content) => content,
            Err(e) => {
                console.debug(&format!(
                    "Cannot fetch image directory listing '{image_dir_url}' ({e})"
                ));
                return None;
            }
        };

        let image_file = util::find_newest_file(
            &image_provider.get_image_file_pattern(name, arch),
            &image_content,
        );

        if let Some(image_file) = &image_file {
            // A timestamped file name is stored as a glob
            let (stored_file, checksum_file) = match image_provider.get_image_file_glob(name, arch)
            {
                Some(glob) => (
                    glob,
                    image_provider.get_checksum_file(Image::IMAGE_FILE, name, arch),
                ),
                None => (
                    image_file.clone(),
                    image_provider.get_checksum_file(image_file, name, arch),
                ),
            };
            web.check_file_exists(&format!("{content_dir_url}{image_file}"))
                .await
                .unwrap_or(false)
                .then(|| Image {
                    distro: image_provider.get_distro().to_string(),
                    display_name: image_provider.get_display_name().to_string(),
                    version: image_provider.get_version(image_file, name),
                    codename: image_provider.get_codename(name),
                    tags: Vec::new(),
                    arch,
                    base_url: content_dir_url,
                    image_file: stored_file,
                    checksum_file,
                    hash_alg: image_provider.get_checksum_alg(),
                    eol: false,
                })
        } else {
            None
        }
    }

    pub fn tag_images(mut images: Vec<Image>) -> Vec<Image> {
        images.sort();

        for image_provider in IMAGE_PROVIDERS {
            for arch in [Arch::AMD64, Arch::ARM64] {
                let is_in_group = |image: &Image| {
                    image.distro == image_provider.get_distro() && image.arch == arch
                };
                let versions = images
                    .iter()
                    .filter(|image| is_in_group(image))
                    .map(|image| image.get_version().to_string())
                    .collect::<Vec<_>>();
                let latest = versions.last().cloned();
                let stable = image_provider.find_stable_version(&versions);
                let end_of_life = image_provider.find_end_of_life_versions(&versions);

                for image in images.iter_mut().filter(|image| is_in_group(image)) {
                    if stable.as_deref() == Some(image.get_version()) {
                        image.tags.push(Image::STABLE_TAG.into());
                    }
                    if latest.as_deref() == Some(image.get_version()) {
                        image.tags.push(Image::LATEST_TAG.into());
                    }
                    image.eol = end_of_life.iter().any(|v| v == image.get_version());
                }
            }
        }

        images
    }

    async fn get_images_from_provider(
        console: &Arc<Console>,
        web: &WebClient,
        image_provider: &dyn image::ImageProvider,
    ) -> Vec<Image> {
        let Ok(content) = web.download_content(image_provider.get_base_url()).await else {
            return Vec::new();
        };

        let mut images = Vec::new();
        for name in image_provider.find_image_names(&content) {
            for arch in [Arch::AMD64, Arch::ARM64] {
                if let Some(image) = Self::get_images_from_provider_name_arch(
                    console,
                    web,
                    image_provider,
                    &name,
                    arch,
                )
                .await
                {
                    images.push(image);
                }
            }
        }
        images
    }

    pub async fn get_images(console: &Arc<Console>) -> Result<Vec<Image>> {
        let web = WebClient::new()?;

        // One task per provider, so each mirror is queried side by side
        let tasks = IMAGE_PROVIDERS
            .iter()
            .map(|image_provider| {
                let (console, web) = (Arc::clone(console), web.clone());
                tokio::spawn(async move {
                    Self::get_images_from_provider(&console, &web, *image_provider).await
                })
            })
            .collect::<Vec<_>>();

        let mut images = Vec::new();
        for (image_provider, task) in IMAGE_PROVIDERS.iter().zip(tasks) {
            // A failed task counts as an empty provider
            let found = task.await.unwrap_or_default();

            // An empty mirror would drop a whole distribution
            if found.is_empty() {
                return Err(Error::EmptyImageProvider(
                    image_provider.get_distro().to_string(),
                ));
            }

            images.extend(found);
        }

        Ok(Self::tag_images(images))
    }

    pub fn serialize(images: &[Image]) -> Result<String> {
        toml::to_string(&ImageList {
            images: images.to_vec(),
        })
        .map_err(Error::from)
    }

    fn find_matching_image(images: &[Image], filter: &ImageName) -> Option<Image> {
        images
            .iter()
            .find(|image| {
                image.distro == filter.get_distro()
                    && image.arch == filter.get_arch()
                    && image.has_name(filter.get_name())
            })
            .cloned()
    }

    pub fn get_all_images() -> Vec<Image> {
        ImageList::read().to_vec()
    }

    pub fn find_image(name: &ImageName) -> Result<Image> {
        Self::find_matching_image(ImageList::read(), name)
            .ok_or_else(|| Error::UnknownImage(name.to_string()))
    }

    pub fn get_display_name(image: &Image) -> String {
        let name = &image.display_name;

        if image.version == Image::ROLLING {
            name.clone()
        } else {
            format!("{name} {}", image.version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HashAlg;
    use std::str::FromStr;

    fn build_image(distro: &str, version: &str, codename: Option<&str>, arch: Arch) -> Image {
        Image {
            distro: distro.to_string(),
            display_name: distro.to_string(),
            version: version.to_string(),
            codename: codename.map(str::to_string),
            tags: Vec::new(),
            arch,
            base_url: "base_url/".to_string(),
            image_file: "image_file".to_string(),
            checksum_file: "checksum_file".to_string(),
            hash_alg: HashAlg::Sha256,
            eol: false,
        }
    }

    #[test]
    fn test_tag_images_tags_the_newest_and_the_newest_lts() {
        let images = vec![
            build_image("ubuntu", "24.04", Some("noble"), Arch::AMD64),
            build_image("ubuntu", "25.10", Some("questing"), Arch::AMD64),
            build_image("ubuntu", "25.04", Some("plucky"), Arch::AMD64),
        ];

        let images = ImageFactory::tag_images(images);

        assert_eq!(images[0].get_tags(), "24.04, noble, stable");
        assert_eq!(images[1].get_tags(), "25.04, plucky");
        assert_eq!(images[2].get_tags(), "25.10, questing, latest");
    }

    #[test]
    fn test_tag_images_tags_each_arch_on_its_own() {
        let images = vec![
            build_image("ubuntu", "24.04", Some("noble"), Arch::AMD64),
            build_image("ubuntu", "26.04", Some("resolute"), Arch::AMD64),
            build_image("ubuntu", "24.04", Some("noble"), Arch::ARM64),
        ];

        let images = ImageFactory::tag_images(images);
        let find_tags = |version: &str, arch: Arch| {
            images
                .iter()
                .find(|image| image.get_version() == version && image.arch == arch)
                .map(|image| image.get_tags())
                .unwrap()
        };

        assert_eq!(find_tags("24.04", Arch::AMD64), "24.04, noble");
        assert_eq!(
            find_tags("26.04", Arch::AMD64),
            "26.04, resolute, stable, latest"
        );
        assert_eq!(
            find_tags("24.04", Arch::ARM64),
            "24.04, noble, stable, latest"
        );
    }

    #[test]
    fn test_tag_images_marks_old_releases_as_end_of_life() {
        let images = vec![
            build_image("debian", "10", Some("buster"), Arch::AMD64),
            build_image("debian", "11", Some("bullseye"), Arch::AMD64),
            build_image("debian", "12", Some("bookworm"), Arch::AMD64),
            build_image("debian", "13", Some("trixie"), Arch::AMD64),
        ];

        let images = ImageFactory::tag_images(images);

        assert_eq!(
            images.iter().map(|image| image.eol).collect::<Vec<_>>(),
            [true, false, false, false]
        );
    }

    #[test]
    fn test_tag_images_tags_a_rolling_release() {
        let images = vec![build_image("archlinux", Image::ROLLING, None, Arch::AMD64)];

        let images = ImageFactory::tag_images(images);

        assert_eq!(images[0].get_image_name(), "archlinux:rolling");
        assert_eq!(images[0].get_tags(), "rolling, stable, latest");
    }

    #[test]
    fn test_get_display_name_adds_the_version_except_for_a_rolling_release() {
        let mut debian = build_image("debian", "13", Some("trixie"), Arch::AMD64);
        debian.display_name = "Debian".to_string();
        let mut archlinux = build_image("archlinux", Image::ROLLING, None, Arch::AMD64);
        archlinux.display_name = "Arch Linux".to_string();

        assert_eq!(ImageFactory::get_display_name(&debian), "Debian 13");
        assert_eq!(ImageFactory::get_display_name(&archlinux), "Arch Linux");
    }

    #[test]
    fn test_find_matching_image_matches_distro_arch_and_name() {
        let images = vec![
            build_image("almalinux", "9", None, Arch::AMD64),
            build_image("debian", "12", Some("bookworm"), Arch::AMD64),
        ];
        let filter = ImageName::from_str("debian:bookworm:amd64").unwrap();

        let found = ImageFactory::find_matching_image(&images, &filter);

        assert_eq!(found, Some(images[1].clone()));
    }

    #[test]
    fn test_find_matching_image_returns_none_on_a_mismatch() {
        let images = vec![build_image("debian", "12", Some("bookworm"), Arch::AMD64)];

        for name in [
            "ubuntu:bookworm:amd64",
            "debian:bookworm:arm64",
            "debian:bullseye:amd64",
        ] {
            let filter = ImageName::from_str(name).unwrap();

            assert_eq!(ImageFactory::find_matching_image(&images, &filter), None);
        }
    }

    #[test]
    fn test_find_image_reads_the_baked_list() {
        let name = ImageName::from_str("ubuntu:stable:amd64").unwrap();

        let image = ImageFactory::find_image(&name).unwrap();

        assert_eq!(image.distro, "ubuntu");
        assert_eq!(image.arch, Arch::AMD64);
    }
}
