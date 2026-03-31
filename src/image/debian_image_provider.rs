use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct DebianImageProvider {}

impl DebianImageProvider {
    fn get_version_from_content(&self, content: &str) -> Option<String> {
        util::find_and_extract(r#"href=\"debian-([0-9]+)-generic-.*.qcow2\""#, content)
            .into_iter()
            .next()
    }
}

impl ImageProvider for DebianImageProvider {
    fn get_vendor(&self) -> String {
        "debian".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://cloud.debian.org/images/cloud/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#"<a href=\"([a-z]+)/\">[a-z]+/</a>"#, content)
    }

    fn get_image_dir_url(&self, name: &str, _arch: Arch) -> String {
        format!("{}{name}/latest/", self.get_image_list_url(),)
    }

    fn get_image_info(&self, content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        self.get_version_from_content(content).map(|version| {
            let base_url = self.get_image_dir_url(name, arch);
            let arch_name = arch.to_string();
            let image_url = format!("{base_url}debian-{version}-generic-{arch_name}.qcow2");
            let checksum_url = format!("{base_url}SHA512SUMS");
            ImageInfo {
                names: vec![version.to_string(), name.to_string()],
                image_url,
                checksum_url,
                hash_alg: HashAlg::Sha512,
            }
        })
    }
}
