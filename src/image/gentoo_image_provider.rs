use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg, Image};
use crate::util;

pub struct GentooImageProvider {}

impl ImageProvider for GentooImageProvider {
    fn get_distro(&self) -> &str {
        "gentoo"
    }

    fn get_base_url(&self) -> &str {
        "https://distfiles.gentoo.org/releases/"
    }

    fn find_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn get_image_dir_path(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!("{arch_name}/autobuilds/current-di-{arch_name}-cloudinit/")
    }

    fn get_version(&self, _image_file: &str, _name: &str) -> String {
        Image::ROLLING.to_string()
    }

    fn get_image_file_glob(&self, arch: Arch) -> Option<String> {
        Some(format!("di-{}-cloudinit-*.qcow2", arch.as_vendor_str()))
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        self.get_image_file_glob(arch)
            .map(|glob| util::convert_glob_to_regex(&glob))
            .unwrap_or_default()
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.sha256")
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
    fn test_get_image_dir_path_uses_distro_arch() {
        assert_eq!(
            GentooImageProvider {}.get_image_dir_path("latest", Arch::AMD64),
            "amd64/autobuilds/current-di-amd64-cloudinit/"
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = GentooImageProvider {}.get_image_file_pattern("latest", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("di-amd64-cloudinit-20250706T165243Z.qcow2")
        );
    }

    #[test]
    fn test_get_checksum_file_appends_suffix() {
        assert_eq!(
            GentooImageProvider {}.get_checksum_file(
                "di-amd64-cloudinit-20250706T165243Z.qcow2",
                "latest",
                Arch::AMD64
            ),
            "di-amd64-cloudinit-20250706T165243Z.qcow2.sha256"
        );
    }
}
