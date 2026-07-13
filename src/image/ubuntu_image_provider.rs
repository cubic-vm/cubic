use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct UbuntuImageProvider {}

impl UbuntuImageProvider {
    fn get_version_from_content(&self, content: &str) -> Option<String> {
        util::find_and_extract(r"ubuntu-([^-]+)-minimal-cloudimg-[^.]+.img", content)
            .into_iter()
            .next()
    }
}

impl ImageProvider for UbuntuImageProvider {
    fn get_vendor(&self) -> &str {
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

    fn get_image_names(&self, image_file: &str, name: &str) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(version) = self.get_version_from_content(image_file) {
            names.push(version);
        }
        names.push(name.to_string());
        names
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
    fn test_get_image_names_extracts_version() {
        assert_eq!(
            UbuntuImageProvider {}
                .get_image_names("ubuntu-24.04-minimal-cloudimg-amd64.img", "noble"),
            ["24.04", "noble"]
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
