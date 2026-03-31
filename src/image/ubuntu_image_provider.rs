use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct UbuntuImageProvider {}

impl UbuntuImageProvider {
    fn get_version_from_content(&self, content: &str) -> Option<String> {
        util::find_and_extract(
            r#"href=\"ubuntu-([0-9]+\.[0-9]+)-minimal-cloudimg-.*.img\""#,
            content,
        )
        .into_iter()
        .next()
    }
}

impl ImageProvider for UbuntuImageProvider {
    fn get_vendor(&self) -> String {
        "ubuntu".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://cloud-images.ubuntu.com/minimal/releases/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#"href=\"([a-z]+)/\""#, content)
    }

    fn get_image_dir_url(&self, name: &str, _arch: Arch) -> String {
        format!("{}{name}/release/", self.get_image_list_url(),)
    }

    fn get_image_info(&self, content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        self.get_version_from_content(content).map(|version| {
            let base_url = self.get_image_dir_url(name, arch);
            let arch_name = arch.to_string();
            let image_url = format!("{base_url}ubuntu-{version}-minimal-cloudimg-{arch_name}.img");
            let checksum_url = format!("{base_url}SHA256SUMS");
            ImageInfo {
                names: vec![version.to_string(), name.to_string()],
                image_url,
                checksum_url,
                hash_alg: HashAlg::Sha256,
            }
        })
    }
}
