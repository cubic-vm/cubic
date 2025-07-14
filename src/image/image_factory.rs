use crate::arch::Arch;
use crate::error::Error;
use crate::image::Image;
use crate::image::ImageCache;
use crate::util;
use crate::web::WebClient;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE_CACHE_LIFETIME_SEC: u64 = 24 * 60 * 60; // = 1 day

struct ImageLocation {
    url: &'static str,
    pattern: LazyLock<Regex>,
    download_url: &'static str,
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
            (Arch::AMD64, ImageLocation {
                url: "https://geo.mirror.pkgbuild.com/images/latest/",
                pattern: LazyLock::new(|| Regex::new(r">(Arch-Linux-x86_64-cloudimg.qcow2)<").unwrap()),
                download_url: "https://geo.mirror.pkgbuild.com/images/(name)/Arch-Linux-x86_64-cloudimg.qcow2",
            }),
            (Arch::ARM64, ImageLocation {
                url: "https://geo.mirror.pkgbuild.com/images/latest/",
                pattern: LazyLock::new(|| Regex::new(r">(Arch-Linux-arm64-cloudimg.qcow2)<").unwrap()),
                download_url: "https://geo.mirror.pkgbuild.com/images/(name)/Arch-Linux-arm64-cloudimg.qcow2",
            })
        ]),
    },


    Distro {
        vendor: "debian",
        name_pattern: "(name)",
        version_pattern: "(version)",
        overview_url: "https://cloud.debian.org/images/cloud/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
        images: HashMap::from([
            (Arch::AMD64, ImageLocation {
                url: "https://cloud.debian.org/images/cloud/(name)/latest/",
                pattern: LazyLock::new(|| Regex::new(r">debian-([0-9]+)-generic-amd64.qcow2<").unwrap()),
                download_url: "https://cloud.debian.org/images/cloud/(name)/latest/debian-(version)-generic-amd64.qcow2",
            }),
            (Arch::ARM64, ImageLocation {
                url: "https://cloud.debian.org/images/cloud/(name)/latest/",
                pattern: LazyLock::new(|| Regex::new(r">debian-([0-9]+)-generic-arm64.qcow2<").unwrap()),
                download_url: "https://cloud.debian.org/images/cloud/(name)/latest/debian-(version)-generic-arm64.qcow2",
            })
        ]),
    },

    Distro {
        vendor: "fedora",
        name_pattern: "(name)",
        version_pattern: "(name)",
        overview_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([4-9][0-9]+)/<").unwrap()),
        images: HashMap::from([
            (Arch::AMD64, ImageLocation {
                url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/x86_64/images/",
                pattern: LazyLock::new(|| Regex::new(r"Fedora-Cloud-Base-Generic-([0-9]+-[0-9]+.[0-9]+).x86_64.qcow2").unwrap()),
                download_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-(version).x86_64.qcow2",
            }),
            (Arch::ARM64, ImageLocation {
                url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/aarch64/images/",
                pattern: LazyLock::new(|| Regex::new(r"Fedora-Cloud-Base-Generic-([0-9]+-[0-9]+.[0-9]+).aarch64.qcow2").unwrap()),
                download_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/aarch64/images/Fedora-Cloud-Base-Generic-(version).aarch64.qcow2",
            })
        ]),
    },

    Distro {
        vendor: "opensuse",
        name_pattern: "(name)",
        version_pattern:  "(name)",
        overview_url: "https://download.opensuse.org/repositories/Cloud:/Images:/",
        overview_pattern: LazyLock::new(|| Regex::new(r">Leap_([0-9]+\.[0-9]+)/<").unwrap()),
        images: HashMap::from([
            (Arch::AMD64, ImageLocation {
                url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/",
                pattern: LazyLock::new(|| Regex::new(r">(openSUSE-Leap-[0-9]+.[0-9]+.x86_64-NoCloud.qcow2)<").unwrap()),
                download_url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/(version)",
            }),
            (Arch::ARM64, ImageLocation {
                url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/",
                pattern: LazyLock::new(|| Regex::new(r">(openSUSE-Leap-[0-9]+.[0-9]+.aarch64-NoCloud.qcow2)<").unwrap()),
                download_url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/(version)",
            })
        ]),
    },

    Distro {
        vendor: "ubuntu",
        name_pattern: "(name)",
        version_pattern:  "(version)",
        overview_url: "https://cloud-images.ubuntu.com/minimal/releases/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
        images: HashMap::from([
            (Arch::AMD64, ImageLocation {
                url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/",
                pattern: LazyLock::new(|| Regex::new(r">ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-amd64.img<").unwrap()),
                download_url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/ubuntu-(version)-minimal-cloudimg-amd64.img",
            }),
            (Arch::ARM64, ImageLocation {
                url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/",
                pattern: LazyLock::new(|| Regex::new(r">ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-arm64.img<").unwrap()),
                download_url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/ubuntu-(version)-minimal-cloudimg-arm64.img",
            })
        ]),
    },
]
});

pub struct ImageFactory;

impl ImageFactory {
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

    fn add_images(web: &mut WebClient, distro: &Distro) -> Vec<Image> {
        let mut images = Vec::new();
        for name in Self::match_content(web, distro.overview_url, &distro.overview_pattern) {
            for (arch, loc) in &distro.images {
                for version in
                    Self::match_content(web, &loc.url.replace("(name)", &name), &loc.pattern)
                {
                    let url = Self::replace_vars(loc.download_url, &name, &version);
                    if let Ok(size) = web.get_file_size(&url) {
                        images.push(Image {
                            vendor: distro.vendor.to_string(),
                            codename: Self::replace_vars(distro.name_pattern, &name, &version),
                            version: Self::replace_vars(distro.version_pattern, &name, &version),
                            arch: *arch,
                            url,
                            size,
                        })
                    }
                }
            }
        }

        images
    }

    fn read_image_cache() -> Option<Vec<Image>> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_secs())
            .unwrap_or_default();

        util::get_image_cache_file()
            .and_then(|path| File::open(path).map_err(Error::Io))
            .and_then(|ref mut reader| ImageCache::deserialize(reader))
            .ok()
            .and_then(|cache| {
                if now - cache.timestamp < IMAGE_CACHE_LIFETIME_SEC {
                    Some(cache.images)
                } else {
                    None
                }
            })
    }

    fn write_image_cache(images: &[Image]) {
        let cache = ImageCache {
            images: images.to_vec(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|time| time.as_secs())
                .unwrap_or_default(),
        };

        // Write cache
        util::get_image_cache_file()
            .and_then(|path| File::create(path).map_err(Error::Io))
            .and_then(|mut file| cache.serialize(&mut file))
            .ok();
    }

    pub fn create_images() -> Result<Vec<Image>, Error> {
        // Read cache
        if let Some(images) = Self::read_image_cache() {
            return Ok(images);
        }

        let web = &mut WebClient::new()?;
        let images: Vec<Image> = DISTROS
            .iter()
            .flat_map(|distro| Self::add_images(web, distro))
            .collect();

        // Write cache
        Self::write_image_cache(&images);

        Ok(images)
    }
}
