use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct UbuntuImageProvider {}

impl UbuntuImageProvider {
    /// Ubuntu releases a long term version every even year in April
    fn is_long_term_version(&self, version: &str) -> bool {
        version.split_once('.').is_some_and(|(year, month)| {
            month == "04" && year.parse::<u32>().is_ok_and(|year| year % 2 == 0)
        })
    }
}

impl ImageProvider for UbuntuImageProvider {
    fn get_distro(&self) -> &str {
        "ubuntu"
    }

    fn get_base_url(&self) -> &str {
        "https://cloud-images.ubuntu.com/minimal/releases/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#"href=\"([a-z]+)/\""#, content)
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("{name}/release/")
    }

    fn get_version(&self, image_file: &str, name: &str) -> String {
        util::find_and_extract(r"ubuntu-([^-]+)-minimal-cloudimg-[^.]+.img", image_file)
            .into_iter()
            .next()
            .unwrap_or_else(|| name.to_string())
    }

    fn get_codename(&self, name: &str) -> Option<String> {
        Some(name.to_string())
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!("ubuntu-[0-9]+\\.[0-9]+-minimal-cloudimg-{arch_name}.img")
    }

    fn get_checksum_file(&self, _image_file: &str, _name: &str, _arch: Arch) -> String {
        "SHA256SUMS".to_string()
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }

    fn find_stable_version(&self, versions: &[String]) -> Option<String> {
        versions
            .iter()
            .rev()
            .find(|version| self.is_long_term_version(version))
            .cloned()
            .or_else(|| versions.last().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_find_image_names_in_listing() {
        let listing = r#"<a href="jammy/">jammy/</a>
<a href="noble/">noble/</a>"#;

        assert_eq!(
            UbuntuImageProvider {}.find_image_names(listing),
            ["jammy", "noble"]
        );
    }

    #[test]
    fn test_get_version_and_codename() {
        let provider = UbuntuImageProvider {};

        assert_eq!(
            provider.get_version("ubuntu-24.04-minimal-cloudimg-amd64.img", "noble"),
            "24.04"
        );
        assert_eq!(provider.get_codename("noble"), Some("noble".to_string()));
    }

    #[test]
    fn test_find_stable_version_picks_the_newest_lts() {
        let provider = UbuntuImageProvider {};

        assert_eq!(
            provider.find_stable_version(
                &["24.04", "24.10", "25.04", "25.10", "26.04"].map(String::from)
            ),
            Some("26.04".to_string())
        );
        assert_eq!(
            provider.find_stable_version(&["25.04", "25.10"].map(String::from)),
            Some("25.10".to_string())
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = UbuntuImageProvider {}.get_image_file_pattern("noble", Arch::ARM64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("ubuntu-24.04-minimal-cloudimg-arm64.img")
        );
    }
}
