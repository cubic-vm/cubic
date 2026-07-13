use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct OpenSuseImageProvider {}

impl ImageProvider for OpenSuseImageProvider {
    fn get_vendor(&self) -> &str {
        "opensuse"
    }

    fn get_base_url(&self) -> &str {
        "https://download.opensuse.org/repositories/Cloud:/Images:/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">Leap_([0-9]+\.[0-9]+)/<"#, content)
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("Leap_{name}/images/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("openSUSE-Leap-{name}.{arch_name}-NoCloud.qcow2")
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
    fn test_find_image_names_extracts_leap_versions() {
        let listing = r#"<a href="Leap_15.5/">Leap_15.5/</a>
<a href="Leap_15.6/">Leap_15.6/</a>
<a href="Images/">Images/</a>"#;

        assert_eq!(
            OpenSuseImageProvider {}.find_image_names(listing),
            ["15.5", "15.6"]
        );
    }

    #[test]
    fn test_get_image_dir_path_restores_leap_prefix() {
        assert_eq!(
            OpenSuseImageProvider {}.get_image_dir_path("15.6", Arch::AMD64),
            "Leap_15.6/images/"
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = OpenSuseImageProvider {}.get_image_file_pattern("15.6", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("openSUSE-Leap-15.6.x86_64-NoCloud.qcow2")
        );
    }
}
