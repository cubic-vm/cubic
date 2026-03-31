use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct OpenSuseImageProvider {}

impl ImageProvider for OpenSuseImageProvider {
    fn get_vendor(&self) -> String {
        "opensuse".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://download.opensuse.org/repositories/Cloud:/Images:/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">Leap_([0-9]+\.[0-9]+)/<"#, content)
    }

    fn get_image_dir_url(&self, name: &str, _arch: Arch) -> String {
        let base_url = self.get_image_list_url();
        format!("{base_url}Leap_{name}/images/",)
    }

    fn get_image_info(&self, _content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        let base_url = self.get_image_dir_url(name, arch);
        let arch_name = arch.as_canonical_str();
        let image_url = format!("{base_url}openSUSE-Leap-{name}.{arch_name}-NoCloud.qcow2");
        let checksum_url = format!("{image_url}.sha256");
        Some(ImageInfo {
            names: vec![name.to_string()],
            image_url,
            checksum_url,
            hash_alg: HashAlg::Sha256,
        })
    }
}
