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
pub struct Image {
    pub vendor: String,
    pub names: Vec<String>,
    pub arch: Arch,
    pub image_url: String,
    pub checksum_url: String,
    pub hash_alg: HashAlg,
    pub size: Option<u64>,
}

impl Image {
    pub fn get_version(&self) -> &str {
        &self.names[0]
    }

    pub fn get_name(&self) -> &str {
        if self.names.len() > 1 {
            &self.names[1]
        } else {
            &self.names[0]
        }
    }

    pub fn get_image_names(&self) -> String {
        format!(
            "{}:{}",
            self.vendor,
            if self.names.len() > 1 {
                format!("{{{}}}", self.names.join(", "))
            } else {
                self.names[0].clone()
            }
        )
    }

    pub fn to_name(&self) -> String {
        format!("{}:{}:{}", self.vendor, self.get_version(), self.arch)
    }

    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.vendor, self.get_name(), self.arch)
    }
}

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut result = self.vendor.cmp(&other.vendor);

        if result == Ordering::Equal {
            let a = &self.names[0];
            let b = &other.names[0];

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

    fn build_image(vendor: &str, names: &[&str]) -> Image {
        Image {
            vendor: vendor.to_string(),
            names: names.iter().map(|name| name.to_string()).collect(),
            arch: Arch::AMD64,
            image_url: String::new(),
            checksum_url: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
        }
    }

    #[test]
    fn test_get_version_is_first_name() {
        assert_eq!(
            build_image("debian", &["12", "bookworm"]).get_version(),
            "12"
        );
    }

    #[test]
    fn test_get_name_prefers_second_name() {
        assert_eq!(
            build_image("debian", &["12", "bookworm"]).get_name(),
            "bookworm"
        );
    }

    #[test]
    fn test_get_name_falls_back_to_single_name() {
        assert_eq!(build_image("debian", &["bookworm"]).get_name(), "bookworm");
    }

    #[test]
    fn test_get_image_names_joins_multiple_names() {
        assert_eq!(
            build_image("debian", &["12", "bookworm"]).get_image_names(),
            "debian:{12, bookworm}"
        );
    }

    #[test]
    fn test_get_image_names_with_single_name() {
        assert_eq!(
            build_image("debian", &["bookworm"]).get_image_names(),
            "debian:bookworm"
        );
    }

    #[test]
    fn test_to_name_uses_version_and_arch() {
        assert_eq!(
            build_image("debian", &["12", "bookworm"]).to_name(),
            "debian:12:amd64"
        );
    }

    #[test]
    fn test_to_file_name_uses_name_and_arch() {
        assert_eq!(
            build_image("debian", &["12", "bookworm"]).to_file_name(),
            "debian_bookworm_amd64"
        );
    }

    #[test]
    fn test_ord_compares_numeric_versions_numerically() {
        assert!(build_image("debian", &["10"]) > build_image("debian", &["9"]));
    }

    #[test]
    fn test_ord_falls_back_to_lexical_comparison() {
        assert!(build_image("ubuntu", &["noble"]) > build_image("ubuntu", &["jammy"]));
    }

    #[test]
    fn test_ord_compares_vendor_first() {
        assert!(build_image("alma", &["9"]) < build_image("debian", &["1"]));
    }
}
