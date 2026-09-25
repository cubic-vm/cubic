use crate::models::Arch;
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, Ordering};
use std::fmt;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlg {
    Sha512,
    Sha256,
}

impl fmt::Display for HashAlg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            HashAlg::Sha512 => "sha512",
            HashAlg::Sha256 => "sha256",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub distro: String,
    pub version: String,
    pub codename: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub arch: Arch,
    pub base_url: String,
    /// A `*` marks a timestamped name resolved at download time
    pub image_file: String,
    /// `{image_file}` stands for the image file name
    pub checksum_file: String,
    pub hash_alg: HashAlg,
    pub size: Option<u64>,
}

impl Image {
    /// Tags that every distribution has
    pub const LATEST_TAG: &'static str = "latest";
    pub const STABLE_TAG: &'static str = "stable";
    /// Version of a rolling release
    pub const ROLLING: &'static str = "rolling";
    /// Placeholder for the image file name in the checksum file name
    pub const IMAGE_FILE: &'static str = "{image_file}";

    pub fn get_version(&self) -> &str {
        &self.version
    }

    /// Codename of a release, or its version when it has none
    pub fn get_name(&self) -> &str {
        self.codename.as_deref().unwrap_or(&self.version)
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.version == name
            || self.codename.as_deref() == Some(name)
            || self.tags.iter().any(|tag| tag == name)
    }

    /// Image name for display, such as debian:12 or archlinux:rolling
    pub fn get_image_name(&self) -> String {
        format!("{}:{}", self.distro, self.version)
    }

    /// Other names of this image, ordered codename, stable, latest
    pub fn get_tags(&self) -> String {
        self.codename
            .iter()
            .chain(self.tags.iter())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn get_image_url(&self) -> String {
        format!("{}{}", self.base_url, self.image_file)
    }

    pub fn get_checksum_url(&self) -> String {
        let checksum_file = self
            .checksum_file
            .replace(Self::IMAGE_FILE, &self.image_file);
        format!("{}{checksum_file}", self.base_url)
    }

    pub fn has_image_file_pattern(&self) -> bool {
        self.image_file.contains('*')
    }

    pub fn to_name(&self) -> String {
        format!("{}:{}", self.get_image_name(), self.arch)
    }

    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.distro, self.get_name(), self.arch)
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut result = self.distro.cmp(&other.distro);

        if result == Ordering::Equal {
            let a = self.get_version();
            let b = other.get_version();

            if let Ok(a) = a.parse::<u32>()
                && let Ok(b) = b.parse::<u32>()
            {
                result = a.cmp(&b);
            } else {
                result = a.cmp(b);
            }
        }

        result
    }
}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_image(distro: &str, version: &str, codename: Option<&str>) -> Image {
        Image {
            distro: distro.to_string(),
            version: version.to_string(),
            codename: codename.map(str::to_string),
            tags: Vec::new(),
            arch: Arch::AMD64,
            base_url: String::new(),
            image_file: String::new(),
            checksum_file: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
        }
    }

    #[test]
    fn test_get_image_name_uses_the_version() {
        assert_eq!(
            build_image("debian", "12", Some("bookworm")).get_image_name(),
            "debian:12"
        );
        assert_eq!(
            build_image("archlinux", Image::ROLLING, None).get_image_name(),
            "archlinux:rolling"
        );
    }

    #[test]
    fn test_get_tags_joins_the_codename_and_the_derived_tags() {
        let mut ubuntu = build_image("ubuntu", "26.04", Some("resolute"));
        ubuntu.tags = vec!["stable".to_string(), "latest".to_string()];

        assert_eq!(ubuntu.get_tags(), "resolute, stable, latest");
        assert_eq!(build_image("fedora", "43", None).get_tags(), "");
    }

    #[test]
    fn test_to_file_name_uses_name_and_arch() {
        assert_eq!(
            build_image("debian", "12", Some("bookworm")).to_file_name(),
            "debian_bookworm_amd64"
        );
        assert_eq!(
            build_image("archlinux", Image::ROLLING, None).to_file_name(),
            "archlinux_rolling_amd64"
        );
    }

    #[test]
    fn test_ord_sorts_by_distro_then_version() {
        assert!(build_image("debian", "10", None) > build_image("debian", "9", None));
        assert!(build_image("ubuntu", "26.04", None) > build_image("ubuntu", "25.10", None));
        assert!(build_image("alma", "9", None) < build_image("debian", "1", None));
    }
}
