use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct RockyLinuxImageProvider {}

impl ImageProvider for RockyLinuxImageProvider {
    fn get_vendor(&self) -> &str {
        "rockylinux"
    }

    fn get_base_url(&self) -> &str {
        "https://dl.rockylinux.org/pub/rocky/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}/images/{arch_name}/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("Rocky-{name}-GenericCloud-Base.latest.{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.CHECKSUM")
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
        let listing = r#"<a href="8/">8/</a>
<a href="9/">9/</a>
<a href="sig/">sig/</a>"#;

        assert_eq!(
            RockyLinuxImageProvider {}.find_image_names(listing),
            ["8", "9"]
        );
    }

    #[test]
    fn test_get_image_dir_path_uses_canonical_arch() {
        assert_eq!(
            RockyLinuxImageProvider {}.get_image_dir_path("9", Arch::ARM64),
            "9/images/aarch64/"
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = RockyLinuxImageProvider {}.get_image_file_pattern("9", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("Rocky-9-GenericCloud-Base.latest.x86_64.qcow2")
        );
    }

    #[test]
    fn test_get_checksum_file_appends_suffix() {
        assert_eq!(
            RockyLinuxImageProvider {}.get_checksum_file(
                "Rocky-9-GenericCloud-Base.latest.x86_64.qcow2",
                "9",
                Arch::AMD64
            ),
            "Rocky-9-GenericCloud-Base.latest.x86_64.qcow2.CHECKSUM"
        );
    }
}
