use crate::arch::Arch;
use crate::env::Environment;
use crate::error::{Error, Result};
use crate::image::{self, Image, ImageCache, ImageName};
use crate::util;
use crate::web::WebClient;

const IMAGE_PROVIDERS: &[&dyn image::ImageProvider] = &[
    &image::AlmaLinuxImageProvider {},
    &image::ArchLinuxImageProvider {},
    &image::DebianImageProvider {},
    &image::FedoraImageProvider {},
    &image::GentooImageProvider {},
    &image::OpenSuseImageProvider {},
    &image::RockyLinuxImageProvider {},
    &image::UbuntuImageProvider {},
];

pub struct ImageFactory {
    env: Environment,
}

impl ImageFactory {
    pub fn new(env: &Environment) -> Self {
        Self { env: env.clone() }
    }

    fn filter_arch(filter: Option<ImageName>) -> Vec<Arch> {
        let mut arches = vec![Arch::AMD64, Arch::ARM64];

        if let Some(filter) = filter {
            arches.retain(|a| filter.get_arch() == *a);
        }

        arches
    }

    fn get_images_from_provider_name_arch(
        web: &mut WebClient,
        image_provider: &dyn image::ImageProvider,
        name: &str,
        arch: Arch,
        filter: Option<ImageName>,
    ) -> Option<Image> {
        let image_dir_url = format!(
            "{}{}",
            image_provider.get_base_url(),
            &image_provider.get_image_dir_path(name, arch)
        );
        let image_content = web.download_content(&image_dir_url).unwrap();

        let image_file = util::find_and_extract(
            &format!(
                "href=\"\\.?/?({})\"",
                image_provider.get_image_file_pattern(name, arch)
            ),
            &image_content,
        );
        let image_file = image_file.first();

        if let Some(image_file) = image_file {
            let image_url = format!("{image_dir_url}{image_file}");
            let names = image_provider.get_image_names(image_file, name);
            web.get_file_size(&image_url)
                .ok()
                .and_then(|size| size)
                .and_then(|size| {
                    if filter
                        .map(|name| names.contains(&name.get_name().to_string()))
                        .unwrap_or(true)
                    {
                        Some(Image {
                            vendor: image_provider.get_vendor().to_string(),
                            names,
                            arch,
                            image_url,
                            checksum_url: format!(
                                "{image_dir_url}{}",
                                image_provider.get_checksum_file(image_file, name, arch)
                            ),
                            hash_alg: image_provider.get_checksum_alg(),
                            size: Some(size),
                        })
                    } else {
                        None
                    }
                })
        } else {
            None
        }
    }

    fn get_images_from_provider(
        web: &mut WebClient,
        image_provider: &dyn image::ImageProvider,
        filter: Option<ImageName>,
    ) -> Vec<Image> {
        web.download_content(image_provider.get_base_url())
            .map(|content| {
                image_provider
                    .find_image_names(&content)
                    .into_iter()
                    .flat_map(|name| {
                        Self::filter_arch(filter.clone())
                            .into_iter()
                            .flat_map(|arch| {
                                Self::get_images_from_provider_name_arch(
                                    web,
                                    image_provider,
                                    &name,
                                    arch,
                                    filter.clone(),
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_images(web: &mut WebClient, filter: Option<ImageName>) -> Vec<Image> {
        let mut images = IMAGE_PROVIDERS
            .iter()
            .filter(|p| filter.is_none() || filter.as_ref().unwrap().get_vendor() == p.get_vendor())
            .flat_map(|provider| Self::get_images_from_provider(web, *provider, filter.clone()))
            .collect::<Vec<_>>();
        images.sort();
        images
    }

    fn read_images(&self, filter: Option<ImageName>) -> Result<Vec<Image>> {
        // Read cache
        let cache = ImageCache::read_from_file(&self.env.get_image_cache_file());

        // Use cache if valid
        if let Some(cache) = &cache
            && cache.is_valid()
        {
            if let Some(name) = filter {
                if let Some(image) = cache.images.iter().find(|image| {
                    (image.vendor == name.get_vendor())
                        && (image.arch == name.get_arch())
                        && image.names.contains(&name.get_name().to_string())
                }) {
                    return Ok(vec![image.clone()]);
                } else {
                    return Ok(Vec::new());
                }
            } else {
                return Ok(cache.images.clone());
            }
        }

        // Fetch image info
        let images = Self::get_images(&mut WebClient::new()?, filter.clone());

        // Return cache if fetching failed
        Ok(
            if images.is_empty()
                && let Some(cache) = &cache
            {
                cache.images.clone()
            } else {
                // Write cache
                if filter.is_none() {
                    ImageCache::new(images.clone()).write_to_file(&self.env.get_image_cache_file());
                }
                images
            },
        )
    }

    pub fn get_all_images(&self) -> Result<Vec<Image>> {
        self.read_images(None)
    }

    pub fn find_image(&self, name: &ImageName) -> Result<Image> {
        self.read_images(Some(name.clone())).and_then(|images| {
            images
                .into_iter()
                .next()
                .ok_or(Error::UnknownImage(name.to_string()))
        })
    }
}
