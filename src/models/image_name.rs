use crate::models::{Arch, Image};
use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

static IMAGE_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("^(?<distro>[\\w-]+)(:(?<name>[\\w\\.]+))?(:(?<arch>amd64|arm64))?$").unwrap()
});

#[derive(Clone, Debug)]
pub struct ImageName {
    distro: String,
    name: String,
    arch: Arch,
}

impl ImageName {
    pub fn get_distro(&self) -> &str {
        &self.distro
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_arch(&self) -> Arch {
        self.arch
    }
}

impl FromStr for ImageName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        IMAGE_NAME_REGEX
            .captures(name)
            .map(|captures| Self {
                distro: captures["distro"].to_string(),
                // A bare distro is a shortcut for the stable release
                name: captures
                    .name("name")
                    .map(|name| name.as_str().to_string())
                    .unwrap_or(Image::STABLE_TAG.to_string()),
                arch: captures
                    .name("arch")
                    .and_then(|arch| Arch::from_str(arch.as_str()).ok())
                    .unwrap_or(Arch::get_host()),
            })
            .ok_or_else(|| {
                "Image name must have the format: distro[:name][:arch] (e.g. debian, debian:bookworm, debian:stable:amd64)"
                    .to_string()
            })
    }
}

impl fmt::Display for ImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}:{}:{}", self.distro, self.name, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_distro_name_and_arch() {
        let image = ImageName::from_str("debian:bookworm").unwrap();
        assert_eq!(image.get_distro(), "debian");
        assert_eq!(image.get_name(), "bookworm");
        assert_eq!(image.get_arch(), Arch::get_host());

        let image = ImageName::from_str("debian:bookworm:arm64").unwrap();
        assert_eq!(image.get_name(), "bookworm");
        assert_eq!(image.get_arch(), Arch::ARM64);
    }

    #[test]
    fn test_parse_distro_with_hyphen() {
        let image = ImageName::from_str("opensuse-leap:16.0:arm64").unwrap();
        assert_eq!(image.get_distro(), "opensuse-leap");
        assert_eq!(image.get_name(), "16.0");
        assert_eq!(image.get_arch(), Arch::ARM64);
    }

    #[test]
    fn test_bare_distro_is_stable() {
        let image = ImageName::from_str("debian").unwrap();
        assert_eq!(image.get_distro(), "debian");
        assert_eq!(image.get_name(), "stable");
        assert_eq!(image.get_arch(), Arch::get_host());
    }

    #[test]
    fn test_reject_unknown_arch() {
        assert!(ImageName::from_str("debian:bookworm:mips").is_err());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(
            ImageName::from_str("debian:bookworm:arm64")
                .unwrap()
                .to_string(),
            "debian:bookworm:arm64"
        );
    }
}
