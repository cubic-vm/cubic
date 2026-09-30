use crate::image::ImageProvider;
use crate::models::{Arch, HashAlg};
use crate::util;
use std::cmp::Ordering;

pub struct OpenSuseLeapImageProvider {}

impl OpenSuseLeapImageProvider {
    /// Oldest release whose checksum file names the unversioned image
    const OLDEST_VERSION: &str = "15.6";
    /// Newest final release, 16.1 is still a pre-release
    const NEWEST_VERSION: &str = "16.0";
}

impl ImageProvider for OpenSuseLeapImageProvider {
    fn get_distro(&self) -> &str {
        "opensuse-leap"
    }

    fn get_base_url(&self) -> &str {
        "https://download.opensuse.org/distribution/leap/"
    }

    fn get_content_url(&self) -> &str {
        "https://downloadcontent.opensuse.org/distribution/leap/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#"href="\./([0-9]+\.[0-9]+)/""#, content)
            .into_iter()
            .filter(|version| {
                util::compare_natural(version, Self::OLDEST_VERSION) != Ordering::Less
                    && util::compare_natural(version, Self::NEWEST_VERSION) != Ordering::Greater
            })
            .collect()
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("{name}/appliances/")
    }

    /// Leap 16 drops the `openSUSE-` prefix from the file name
    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let name = regex::escape(name);
        let arch_name = arch.as_canonical_str();
        format!("(?:openSUSE-)?Leap-{name}-Minimal-VM\\.{arch_name}-Cloud\\.qcow2")
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

    #[test]
    fn test_find_image_names_keeps_supported_releases() {
        let listing = r#"<a href="../">../</a>
<a href="./15.5/">15.5/</a>
<a href="./15.6/">15.6/</a>
<a href="./16.0/">16.0/</a>
<a href="./16.1/">16.1/</a>"#;

        assert_eq!(
            OpenSuseLeapImageProvider {}.find_image_names(listing),
            ["15.6", "16.0"]
        );
    }

    #[test]
    fn test_get_image_dir_path_points_to_appliances() {
        assert_eq!(
            OpenSuseLeapImageProvider {}.get_image_dir_path("16.0", Arch::AMD64),
            "16.0/appliances/"
        );
    }

    #[test]
    fn test_image_file_pattern_matches_both_name_styles() {
        let provider = OpenSuseLeapImageProvider {};
        let listing = r#"<a href="./openSUSE-Leap-15.6-Minimal-VM.x86_64-Cloud.qcow2">
<a href="./Leap-16.0-Minimal-VM.aarch64-Cloud-Build18.68.qcow2">
<a href="./Leap-16.0-Minimal-VM.aarch64-Cloud.qcow2">
<a href="./Leap-16.0-Minimal-VM.aarch64-kvm.qcow2">"#;
        let find = |name, arch| {
            util::find_newest_file(&provider.get_image_file_pattern(name, arch), listing)
        };

        assert_eq!(
            find("15.6", Arch::AMD64).as_deref(),
            Some("openSUSE-Leap-15.6-Minimal-VM.x86_64-Cloud.qcow2")
        );
        assert_eq!(
            find("16.0", Arch::ARM64).as_deref(),
            Some("Leap-16.0-Minimal-VM.aarch64-Cloud.qcow2")
        );
    }
}
