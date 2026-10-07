use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg, Image};

pub struct OpenSuseTumbleweedImageProvider {}

impl ImageProvider for OpenSuseTumbleweedImageProvider {
    fn get_distro(&self) -> &str {
        "opensuse-tumbleweed"
    }

    fn get_display_name(&self) -> &str {
        "openSUSE Tumbleweed"
    }

    fn get_base_url(&self) -> &str {
        "https://download.opensuse.org/"
    }

    fn get_content_url(&self) -> &str {
        "https://downloadcontent.opensuse.org/"
    }

    fn find_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    /// ARM64 is a port and lives in its own directory tree
    fn get_image_dir_path(&self, _name: &str, arch: Arch) -> String {
        match arch {
            Arch::AMD64 => "tumbleweed/appliances/".to_string(),
            Arch::ARM64 => "ports/aarch64/tumbleweed/appliances/".to_string(),
        }
    }

    fn get_version(&self, _image_file: &str, _name: &str) -> String {
        Image::ROLLING.to_string()
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("openSUSE-Tumbleweed-Minimal-VM\\.{arch_name}-Cloud\\.qcow2")
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
    use crate::util;

    #[test]
    fn test_image_file_pattern_skips_the_snapshot_file() {
        let listing = r#"<a href="./openSUSE-Tumbleweed-Minimal-VM.x86_64-1.0.0-Cloud-Snapshot20261005.qcow2">
<a href="./openSUSE-Tumbleweed-Minimal-VM.x86_64-Cloud.qcow2">
<a href="./openSUSE-Tumbleweed-Minimal-VM.x86_64-Cloud.qcow2.sha256">"#;
        let pattern =
            OpenSuseTumbleweedImageProvider {}.get_image_file_pattern("latest", Arch::AMD64);

        assert_eq!(
            util::find_newest_file(&pattern, listing).as_deref(),
            Some("openSUSE-Tumbleweed-Minimal-VM.x86_64-Cloud.qcow2")
        );
    }
}
