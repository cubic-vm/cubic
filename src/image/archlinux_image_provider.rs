use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};

pub struct ArchLinuxImageProvider {}

impl ImageProvider for ArchLinuxImageProvider {
    fn get_vendor(&self) -> &str {
        "archlinux"
    }

    fn get_base_url(&self) -> &str {
        "https://geo.mirror.pkgbuild.com/images/"
    }

    fn find_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn get_image_dir_path(&self, _name: &str, _arch: Arch) -> String {
        "latest/".to_string()
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("Arch-Linux-{arch_name}-cloudimg.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.SHA256")
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
    fn test_find_image_names_is_always_latest() {
        assert_eq!(ArchLinuxImageProvider {}.find_image_names(""), ["latest"]);
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = ArchLinuxImageProvider {}.get_image_file_pattern("latest", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("Arch-Linux-x86_64-cloudimg.qcow2")
        );
    }

    #[test]
    fn test_get_checksum_file_appends_suffix() {
        assert_eq!(
            ArchLinuxImageProvider {}.get_checksum_file(
                "Arch-Linux-x86_64-cloudimg.qcow2",
                "latest",
                Arch::AMD64
            ),
            "Arch-Linux-x86_64-cloudimg.qcow2.SHA256"
        );
    }
}
