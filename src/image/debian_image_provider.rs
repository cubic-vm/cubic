use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct DebianImageProvider {}

impl ImageProvider for DebianImageProvider {
    fn get_distro(&self) -> &str {
        "debian"
    }

    fn get_display_name(&self) -> &str {
        "Debian"
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

    fn get_version(&self, image_file: &str, name: &str) -> String {
        util::find_and_extract(r"debian-([^-]+)-genericcloud-[^.]+\.qcow2", image_file)
            .into_iter()
            .next()
            .unwrap_or_else(|| name.to_string())
    }

    fn get_codename(&self, name: &str) -> Option<String> {
        Some(name.to_string())
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!(r"debian-[0-9]+-genericcloud-{arch_name}\.qcow2")
    }

    fn get_checksum_file(&self, _image_file: &str, _name: &str, _arch: Arch) -> String {
        "SHA512SUMS".to_string()
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha512
    }

    /// 5 years of support, a release every 2 years
    fn find_end_of_life_versions(&self, versions: &[String]) -> Vec<String> {
        versions[..versions.len().saturating_sub(3)].to_vec()
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
    fn test_get_version_and_codename() {
        let provider = DebianImageProvider {};

        assert_eq!(
            provider.get_version("debian-12-genericcloud-amd64.qcow2", "bookworm"),
            "12"
        );
        assert_eq!(
            provider.get_codename("bookworm"),
            Some("bookworm".to_string())
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = DebianImageProvider {}.get_image_file_pattern("bookworm", Arch::AMD64);
        let regex = Regex::new(&format!("^{pattern}$")).unwrap();

        assert!(regex.is_match("debian-12-genericcloud-amd64.qcow2"));
        assert!(!regex.is_match("debian-12-generic-amd64.qcow2"));
    }
}
