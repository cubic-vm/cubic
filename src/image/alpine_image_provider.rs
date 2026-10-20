use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;
use std::cmp::Ordering;

pub struct AlpineImageProvider {}

impl AlpineImageProvider {
    /// Oldest release with the current image file name
    const OLDEST_VERSION: &str = "3.24";
}

impl ImageProvider for AlpineImageProvider {
    fn get_distro(&self) -> &str {
        "alpine"
    }

    fn get_display_name(&self) -> &str {
        "Alpine Linux"
    }

    fn get_base_url(&self) -> &str {
        "https://dl-cdn.alpinelinux.org/alpine/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">v([0-9]+\.[0-9]+)/<"#, content)
            .into_iter()
            .filter(|version| {
                util::compare_natural(version, Self::OLDEST_VERSION) != Ordering::Less
            })
            .collect()
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("v{name}/releases/cloud/")
    }

    fn get_image_file_glob(&self, name: &str, arch: Arch) -> Option<String> {
        let arch_name = arch.as_canonical_str();
        Some(format!("alpine-{name}.*-{arch_name}-cloudinit-r0.qcow2"))
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.sha512")
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha512
    }

    /// 2 years of support, a release every 6 months
    fn find_end_of_life_versions(&self, versions: &[String]) -> Vec<String> {
        versions[..versions.len().saturating_sub(4)].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_find_image_names_keeps_supported_releases() {
        let listing = r#"<a href="edge/">edge/</a>
<a href="latest-stable/">latest-stable/</a>
<a href="v3.9/">v3.9/</a>
<a href="v3.20/">v3.20/</a>
<a href="v3.23/">v3.23/</a>
<a href="v3.24/">v3.24/</a>
<a href="v3.25/">v3.25/</a>"#;

        assert_eq!(
            AlpineImageProvider {}.find_image_names(listing),
            ["3.24", "3.25"]
        );
    }

    #[test]
    fn test_image_file_pattern_matches_generic_cloud_init_image() {
        let pattern = AlpineImageProvider {}.get_image_file_pattern("3.24", Arch::AMD64);
        let regex = Regex::new(&format!("^{pattern}$")).unwrap();

        assert!(regex.is_match("alpine-3.24.2-x86_64-cloudinit-r0.qcow2"));
        assert!(regex.is_match("alpine-3.24.10-x86_64-cloudinit-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.24.2-x86_64-cloudinit-metal-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.24.2-x86_64-tiny-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.24.2-aarch64-cloudinit-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.240.2-x86_64-cloudinit-r0.qcow2"));
    }
}
