use crate::error::Error;
use crate::image::Image;
use crate::web::WebClient;
use regex::Regex;

use std::sync::LazyLock;

struct Distro {
    vendor: &'static str,
    name_pattern: &'static str,
    version_pattern: &'static str,
    overview_url: &'static str,
    overview_pattern: LazyLock<Regex>,
    image_url: &'static str,
    image_pattern: LazyLock<Regex>,
    download_url: &'static str,
}

static DISTROS: LazyLock<Vec<Distro>> = LazyLock::new(|| {
    vec![
    Distro {
        vendor: "archlinux",
        name_pattern: "(name)",
        version_pattern: "(name)",
        overview_url: "https://geo.mirror.pkgbuild.com/images/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
        image_url: "https://geo.mirror.pkgbuild.com/images/latest/",
        image_pattern: LazyLock::new(|| Regex::new(r">(Arch-Linux-x86_64-cloudimg.qcow2)<").unwrap()),
        download_url: "https://geo.mirror.pkgbuild.com/images/(name)/Arch-Linux-x86_64-cloudimg.qcow2",
    },


    Distro {
        vendor: "debian",
        name_pattern: "(name)",
        version_pattern: "(version)",
        overview_url: "https://cloud.debian.org/images/cloud/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
        image_url: "https://cloud.debian.org/images/cloud/(name)/latest/",
        image_pattern: LazyLock::new(|| Regex::new(r">debian-([0-9]+)-generic-amd64.qcow2<").unwrap()),
        download_url: "https://cloud.debian.org/images/cloud/(name)/latest/debian-(version)-generic-amd64.qcow2",
    },

    Distro {
        vendor: "fedora",
        name_pattern: "(name)",
        version_pattern: "(name)",
        overview_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([0-9]+)/<").unwrap()),
        image_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/x86_64/images/",
        image_pattern: LazyLock::new(|| Regex::new(r"Fedora-Cloud-Base-Generic-([0-9]+-[0-9]+.[0-9]+).x86_64.qcow2").unwrap()),
        download_url: "https://download.fedoraproject.org/pub/fedora/linux/releases/(name)/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-(version).x86_64.qcow2",
    },

    Distro {
        vendor: "ubuntu",
        name_pattern: "(name)",
        version_pattern:  "(version)",
        overview_url: "https://cloud-images.ubuntu.com/minimal/releases/",
        overview_pattern: LazyLock::new(|| Regex::new(r">([a-z]+)/<").unwrap()),
        image_url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/",
        image_pattern: LazyLock::new(|| Regex::new(r">ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-amd64.img<").unwrap()),
        download_url: "https://cloud-images.ubuntu.com/minimal/releases/(name)/release/ubuntu-(version)-minimal-cloudimg-amd64.img",
    },

    Distro {
        vendor: "opensuse",
        name_pattern: "(name)",
        version_pattern:  "(name)",
        overview_url: "https://download.opensuse.org/repositories/Cloud:/Images:/",
        overview_pattern: LazyLock::new(|| Regex::new(r">Leap_([0-9]+\.[0-9]+)/<").unwrap()),
        image_url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/",
        image_pattern: LazyLock::new(|| Regex::new(r">(openSUSE-Leap-[0-9]+.[0-9]+.x86_64-NoCloud.qcow2)<").unwrap()),
        download_url: "https://download.opensuse.org/repositories/Cloud:/Images:/Leap_15.6/images/(version)",
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
        let names = Self::match_content(web, distro.overview_url, &distro.overview_pattern);
        names
            .iter()
            .flat_map(|name| {
                let versions = Self::match_content(
                    web,
                    &distro.image_url.replace("(name)", name),
                    &distro.image_pattern,
                );
                versions
                    .iter()
                    .map(|version| {
                        let url = Self::replace_vars(distro.download_url, name, version);
                        let size = web.get_file_size(&url).unwrap_or_default();
                        Image {
                            vendor: distro.vendor.to_string(),
                            codename: Self::replace_vars(distro.name_pattern, name, version),
                            version: Self::replace_vars(distro.version_pattern, name, version),
                            url,
                            size,
                        }
                    })
                    .collect::<Vec<Image>>()
            })
            .collect()
    }

    pub fn create_images() -> Result<Vec<Image>, Error> {
        let web = &mut WebClient::new()?;

        Ok(DISTROS
            .iter()
            .flat_map(|distro| Self::add_images(web, distro))
            .collect())
    }

    pub fn create_images_for_distro(name: &str) -> Result<Vec<Image>, Error> {
        let web = &mut WebClient::new()?;

        Ok(DISTROS
            .iter()
            .filter(|distro| distro.vendor == name)
            .flat_map(|distro| Self::add_images(web, distro))
            .collect())
    }
}
