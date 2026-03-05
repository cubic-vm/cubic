use crate::arch::Arch;
use crate::env::Environment;
use crate::error::{Error, Result};
use crate::image::{HashAlg, Image, ImageCache, ImageName};
use crate::web::WebClient;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

struct ImageLocation {
    path: &'static str,
    pattern: LazyLock<Regex>,
    image_name: &'static str,
    checksum_name: &'static str,
    hash_alg: HashAlg,
}

struct Distro {
    vendor: &'static str,
    name_pattern: &'static str,
    version_pattern: &'static str,
    overview_url: &'static str,
    overview_pattern: LazyLock<Regex>,
    images: HashMap<Arch, ImageLocation>,
}

static DISTROS: LazyLock<Vec<Distro>> = LazyLock::new(|| {
    vec![
        Distro {
            vendor: "archlinux",
            name_pattern: "(name)",
            version_pattern: "(name)",
            overview_url: "https://geo.mirror.pkgbuild.com/images/",
            overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "latest/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">(Arch-Linux-x86_64-cloudimg.qcow2)<").unwrap()
                        }),
                        image_name: "Arch-Linux-x86_64-cloudimg.qcow2",
                        checksum_name: "Arch-Linux-x86_64-cloudimg.qcow2.SHA256",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "latest/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">(Arch-Linux-arm64-cloudimg.qcow2)<").unwrap()
                        }),
                        image_name: "Arch-Linux-arm64-cloudimg.qcow2",
                        checksum_name: "Arch-Linux-arm64-cloudimg.qcow2.SHA256",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
            ]),
        },
        Distro {
            vendor: "debian",
            name_pattern: "(name)",
            version_pattern: "(version)",
            overview_url: "https://cloud.debian.org/images/cloud/",
            overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "(name)/latest/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">debian-([0-9]+)-generic-amd64.qcow2<").unwrap()
                        }),
                        image_name: "debian-(version)-generic-amd64.qcow2",
                        checksum_name: "SHA512SUMS",
                        hash_alg: HashAlg::Sha512,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "(name)/latest/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">debian-([0-9]+)-generic-arm64.qcow2<").unwrap()
                        }),
                        image_name: "debian-(version)-generic-arm64.qcow2",
                        checksum_name: "SHA512SUMS",
                        hash_alg: HashAlg::Sha512,
                    },
                ),
            ]),
        },
        Distro {
            vendor: "fedora",
            name_pattern: "(name)",
            version_pattern: "(name)",
            overview_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/",
            overview_pattern: LazyLock::new(|| Regex::new(r">([4-9][0-9]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "(name)/Cloud/x86_64/images/",
                        pattern: LazyLock::new(|| {
                            Regex::new(
                                r"Fedora-Cloud-Base-Generic-([0-9]+-[0-9]+.[0-9]+).x86_64.qcow2",
                            )
                            .unwrap()
                        }),
                        image_name: "Fedora-Cloud-Base-Generic-(version).x86_64.qcow2",
                        checksum_name: "Fedora-Cloud-(version)-x86_64-CHECKSUM",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "(name)/Cloud/aarch64/images/",
                        pattern: LazyLock::new(|| {
                            Regex::new(
                                r"Fedora-Cloud-Base-Generic-([0-9]+-[0-9]+.[0-9]+).aarch64.qcow2",
                            )
                            .unwrap()
                        }),
                        image_name: "Fedora-Cloud-Base-Generic-(version).aarch64.qcow2",
                        checksum_name: "Fedora-Cloud-(version)-aarch64-CHECKSUM",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
            ]),
        },
        Distro {
            vendor: "opensuse",
            name_pattern: "(name)",
            version_pattern: "(name)",
            overview_url: "https://download.opensuse.org/repositories/Cloud:/Images:/",
            overview_pattern: LazyLock::new(|| Regex::new(r">Leap_([0-9]+\.[0-9]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "Leap_(name)/images/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">(openSUSE-Leap-[0-9]+.[0-9]+.x86_64-NoCloud.qcow2)<")
                                .unwrap()
                        }),
                        image_name: "(version)",
                        checksum_name: "(version).sha256",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "Leap_(name)/images/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">(openSUSE-Leap-[0-9]+.[0-9]+.aarch64-NoCloud.qcow2)<")
                                .unwrap()
                        }),
                        image_name: "(version)",
                        checksum_name: "(version).sha256",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
            ]),
        },
        Distro {
            vendor: "rockylinux",
            name_pattern: "(name)",
            version_pattern: "(name)",
            overview_url: "https://dl.rockylinux.org/pub/rocky/",
            overview_pattern: LazyLock::new(|| Regex::new(r">([0-9]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "(name)/images/x86_64/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">Rocky-([0-9]+)-GenericCloud-Base.latest.x86_64.qcow2<")
                                .unwrap()
                        }),
                        image_name: "Rocky-(name)-GenericCloud-Base.latest.x86_64.qcow2",
                        checksum_name: "Rocky-(name)-GenericCloud-Base.latest.x86_64.qcow2.CHECKSUM",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "(name)/images/aarch64/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">Rocky-([0-9]+)-GenericCloud-Base.latest.aarch64.qcow2<")
                                .unwrap()
                        }),
                        image_name: "Rocky-(name)-GenericCloud-Base.latest.aarch64.qcow2",
                        checksum_name: "Rocky-(name)-GenericCloud-Base.latest.aarch64.qcow2.CHECKSUM",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
            ]),
        },
        Distro {
            vendor: "ubuntu",
            name_pattern: "(name)",
            version_pattern: "(version)",
            overview_url: "https://cloud-images.ubuntu.com/minimal/releases/",
            overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
            images: HashMap::from([
                (
                    Arch::AMD64,
                    ImageLocation {
                        path: "(name)/release/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-amd64.img<")
                                .unwrap()
                        }),
                        image_name: "ubuntu-(version)-minimal-cloudimg-amd64.img",
                        checksum_name: "SHA256SUMS",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
                (
                    Arch::ARM64,
                    ImageLocation {
                        path: "(name)/release/",
                        pattern: LazyLock::new(|| {
                            Regex::new(r">ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-arm64.img<")
                                .unwrap()
                        }),
                        image_name: "ubuntu-(version)-minimal-cloudimg-arm64.img",
                        checksum_name: "SHA256SUMS",
                        hash_alg: HashAlg::Sha256,
                    },
                ),
            ]),
        },
    ]
});

pub struct ImageFactory {
    env: Environment,
}

impl ImageFactory {
    pub fn new(env: &Environment) -> Self {
        Self { env: env.clone() }
    }

    fn match_content(web: &mut WebClient, url: &str, pattern: &LazyLock<Regex>) -> Vec<String> {
        web.download_content(url)
            .map(|content| {
                pattern
                    .captures_iter(&content)
                    .map(|content| content.extract::<1>())
                    .map(|(_, values)| values[0].to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn replace_vars(text: &str, name: &str, version: &str) -> String {
        text.replace("(name)", name).replace("(version)", version)
    }

    fn add_images(
        web: &mut WebClient,
        distro: &Distro,
        filter_name: Option<ImageName>,
    ) -> Vec<Image> {
        let mut images = Vec::new();
        for name in Self::match_content(web, distro.overview_url, &distro.overview_pattern) {
            for (arch, loc) in &distro.images {
                if let Some(filter_name) = &filter_name
                    && *arch != filter_name.get_arch()
                {
                    continue;
                }

                let loc_url = format!(
                    "{}{}",
                    distro.overview_url,
                    loc.path.replace("(name)", &name)
                );
                for version in Self::match_content(web, &loc_url, &loc.pattern) {
                    let image_url = format!(
                        "{loc_url}{}",
                        Self::replace_vars(loc.image_name, &name, &version)
                    );
                    let checksum_url = format!(
                        "{loc_url}{}",
                        Self::replace_vars(loc.checksum_name, &name, &version)
                    );
                    let version = Self::replace_vars(distro.version_pattern, &name, &version);
                    let codename = Self::replace_vars(distro.name_pattern, &name, &version);
                    let mut names = vec![version.clone()];
                    if version != codename {
                        names.push(codename);
                    }

                    if let Some(filter_name) = &filter_name
                        && !names.contains(&filter_name.get_name().to_string())
                    {
                        continue;
                    }

                    if let Ok(size) = web.get_file_size(&image_url) {
                        images.push(Image {
                            vendor: distro.vendor.to_string(),
                            names,
                            arch: *arch,
                            image_url,
                            checksum_url,
                            hash_alg: loc.hash_alg,
                            size,
                        })
                    }
                }
            }
        }

        images
    }

    pub fn create_images(&self) -> Result<Vec<Image>> {
        // Read cache
        let cache = ImageCache::read_from_file(&self.env.get_image_cache_file());

        // Use cache if valid
        if let Some(cache) = &cache
            && cache.is_valid()
        {
            return Ok(cache.images.clone());
        }

        // Fetch images
        let web = &mut WebClient::new()?;
        let images: Vec<Image> = DISTROS
            .iter()
            .flat_map(|distro| Self::add_images(web, distro, None))
            .collect();

        // Return cache if fetching failed
        if images.is_empty() {
            if let Some(cache) = &cache {
                return Ok(cache.images.clone());
            }
        } else {
            // Write cache
            ImageCache::new(images.clone()).write_to_file(&self.env.get_image_cache_file());
        }

        Ok(images)
    }

    pub fn get_image(&self, name: &ImageName) -> Result<Image> {
        // Read cache
        let cache = ImageCache::read_from_file(&self.env.get_image_cache_file());

        // Use cache if valid
        if let Some(cache) = &cache
            && cache.is_valid()
            && let Some(image) = cache.images.iter().find(|image| {
                (image.vendor == name.get_vendor())
                    && (image.arch == name.get_arch())
                    && image.names.contains(&name.get_name().to_string())
            })
        {
            return Ok(image.clone());
        }

        // Fetch image
        let web = &mut WebClient::new()?;
        DISTROS
            .iter()
            .filter(|distro| distro.vendor == name.get_vendor())
            .flat_map(|distro| Self::add_images(web, distro, Some(name.clone())))
            .find(|image| {
                (image.vendor == name.get_vendor())
                    && (image.arch == name.get_arch())
                    && image.names.contains(&name.get_name().to_string())
            })
            .ok_or(Error::UnknownImage(name.to_string()))
    }
}
