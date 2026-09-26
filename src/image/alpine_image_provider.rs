use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;
use std::cmp::Ordering;

pub struct AlpineImageProvider {}

impl AlpineImageProvider {
    /// Oldest release that ships a cloud-init image
    const OLDEST_VERSION: &str = "3.21";
}

impl ImageProvider for AlpineImageProvider {
    fn get_distro(&self) -> &str {
        "alpine"
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

    /// Newer releases drop the `-uefi` part from the file name
    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let name = regex::escape(name);
        let arch_name = arch.as_canonical_str();
        format!("alpine-{name}\\.[0-9]+-{arch_name}(?:-uefi)?-cloudinit-r[0-9]+\\.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.sha512")
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
    fn test_find_image_names_keeps_supported_releases() {
        let listing = r#"<a href="edge/">edge/</a>
<a href="latest-stable/">latest-stable/</a>
<a href="v3.9/">v3.9/</a>
<a href="v3.20/">v3.20/</a>
<a href="v3.21/">v3.21/</a>
<a href="v3.24/">v3.24/</a>"#;

        assert_eq!(
            AlpineImageProvider {}.find_image_names(listing),
            ["3.21", "3.24"]
        );
    }

    #[test]
    fn test_image_file_pattern_matches_generic_cloud_init_image() {
        let pattern = AlpineImageProvider {}.get_image_file_pattern("3.23", Arch::AMD64);
        let regex = Regex::new(&format!("^{pattern}$")).unwrap();

        assert!(regex.is_match("alpine-3.23.6-x86_64-uefi-cloudinit-r0.qcow2"));
        assert!(regex.is_match("alpine-3.23.10-x86_64-cloudinit-r1.qcow2"));
        assert!(!regex.is_match("alpine-3.23.6-x86_64-uefi-cloudinit-metal-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.23.6-x86_64-uefi-tiny-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.23.6-aarch64-uefi-cloudinit-r0.qcow2"));
        assert!(!regex.is_match("alpine-3.230.6-x86_64-cloudinit-r0.qcow2"));
    }
}
