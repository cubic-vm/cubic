use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;

pub struct FedoraImageProvider {}

impl ImageProvider for FedoraImageProvider {
    fn get_vendor(&self) -> &str {
        "fedora"
    }

    fn get_base_url(&self) -> &str {
        "https://dl.fedoraproject.org/pub/fedora/linux/releases/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
            .into_iter()
            .filter(|version| version.parse::<u32>().map(|v| v > 40).unwrap_or_default())
            .collect()
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}/Cloud/{arch_name}/images/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("Fedora-Cloud-Base-Generic-{name}-.*.{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        image_file.replace("-Base-Generic", "").replace(
            &format!(".{arch_name}.qcow2"),
            &format!("-{arch_name}-CHECKSUM"),
        )
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
    fn test_find_image_names_keeps_versions_above_forty() {
        let listing = r#"<a href="39/">39/</a>
<a href="40/">40/</a>
<a href="41/">41/</a>
<a href="42/">42/</a>
<a href="test/">test/</a>"#;

        assert_eq!(
            FedoraImageProvider {}.find_image_names(listing),
            ["41", "42"]
        );
    }

    #[test]
    fn test_get_image_dir_path_uses_canonical_arch() {
        assert_eq!(
            FedoraImageProvider {}.get_image_dir_path("42", Arch::ARM64),
            "42/Cloud/aarch64/images/"
        );
    }

    #[test]
    fn test_image_file_pattern_matches_image_file() {
        let pattern = FedoraImageProvider {}.get_image_file_pattern("42", Arch::AMD64);

        assert!(
            Regex::new(&pattern)
                .unwrap()
                .is_match("Fedora-Cloud-Base-Generic-42-1.1.x86_64.qcow2")
        );
    }

    #[test]
    fn test_get_checksum_file_rewrites_image_file() {
        assert_eq!(
            FedoraImageProvider {}.get_checksum_file(
                "Fedora-Cloud-Base-Generic-42-1.1.x86_64.qcow2",
                "42",
                Arch::AMD64
            ),
            "Fedora-Cloud-42-1.1-x86_64-CHECKSUM"
        );
    }
}
