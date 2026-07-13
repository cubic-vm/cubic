use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct DebianImageProvider {}

impl DebianImageProvider {
    fn get_version_from_content(&self, content: &str) -> Option<String> {
        util::find_and_extract(r"debian-([^-]+)-generic-[^.]+.qcow2", content)
            .into_iter()
            .next()
    }
}

impl ImageProvider for DebianImageProvider {
    fn get_vendor(&self) -> &str {
        "debian"
    }

    fn get_base_url(&self) -> &str {
        "https://cloud.debian.org/images/cloud/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#"<a href=\"([a-z]+)/\">[a-z]+/</a>"#, content)
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("{name}/latest/")
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
        format!("debian-[0-9]+-generic-{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, _image_file: &str, _name: &str, _arch: Arch) -> String {
        "SHA512SUMS".to_string()
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha512
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_find_image_names_in_listing() {
        let listing = r#"<a href="bookworm/">bookworm/</a>
<a href="trixie/">trixie/</a>
<a href="OpenStack/">OpenStack/</a>"#;

        assert_eq!(
            DebianImageProvider {}.find_image_names(listing),
            ["bookworm", "trixie"]
        );
    }

    #[test]
    fn test_get_image_names_extracts_version() {
        assert_eq!(
            DebianImageProvider {}.get_image_names("debian-12-generic-amd64.qcow2", "bookworm"),
            ["12", "bookworm"]
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = DebianImageProvider {}.get_image_file_pattern("bookworm", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("debian-12-generic-amd64.qcow2")
        );
    }
}
