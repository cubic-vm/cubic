use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;
use std::cmp::Ordering;

pub struct CentOsImageProvider {}

impl CentOsImageProvider {
    /// Stream 9 publishes no x86_64 checksums
    const OLDEST_VERSION: &str = "10";
}

impl ImageProvider for CentOsImageProvider {
    fn get_distro(&self) -> &str {
        "centos"
    }

    fn get_base_url(&self) -> &str {
        "https://cloud.centos.org/centos/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)-stream/<"#, content)
            .into_iter()
            .filter(|version| {
                util::compare_natural(version, Self::OLDEST_VERSION) != Ordering::Less
            })
            .collect()
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}-stream/{arch_name}/images/")
    }

    /// Builds are dated and a `latest` copy is not always there
    fn get_image_file_glob(&self, name: &str, arch: Arch) -> Option<String> {
        let arch_name = arch.as_canonical_str();
        Some(format!(
            "CentOS-Stream-GenericCloud-{name}-*.{arch_name}.qcow2"
        ))
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.SHA256SUM")
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
    fn test_find_image_names_keeps_supported_releases() {
        let listing = r#"<a href="7/">7/</a>
<a href="8-stream/">8-stream/</a>
<a href="9-stream/">9-stream/</a>
<a href="10-stream/">10-stream/</a>"#;

        assert_eq!(CentOsImageProvider {}.find_image_names(listing), ["10"]);
    }

    #[test]
    fn test_image_file_pattern_matches_generic_cloud_image() {
        let pattern = CentOsImageProvider {}.get_image_file_pattern("10", Arch::AMD64);
        let regex = Regex::new(&format!("^{pattern}$")).unwrap();

        assert!(regex.is_match("CentOS-Stream-GenericCloud-10-20260922.0.x86_64.qcow2"));
        assert!(regex.is_match("CentOS-Stream-GenericCloud-10-latest.x86_64.qcow2"));
        assert!(!regex.is_match("CentOS-Stream-GenericCloud-x86_64-10-20260728.1.x86_64.qcow2"));
        assert!(!regex.is_match("CentOS-Stream-GenericCloud-10-20260922.0.aarch64.qcow2"));
        assert!(!regex.is_match("CentOS-Stream-GenericCloud-9-20260922.0.x86_64.qcow2"));
    }
}
