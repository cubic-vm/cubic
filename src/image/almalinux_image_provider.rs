use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct AlmaLinuxImageProvider {}

impl ImageProvider for AlmaLinuxImageProvider {
    fn get_vendor(&self) -> String {
        "almalinux".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://raw.repo.almalinux.org/almalinux/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
    }

    fn get_image_dir_url(&self, name: &str, arch: Arch) -> String {
        let base_url = self.get_image_list_url();
        let arch_name = arch.as_canonical_str();
        format!("{base_url}{name}/cloud/{arch_name}/images/",)
    }

    fn get_image_info(&self, _content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        let base_url = self.get_image_dir_url(name, arch);
        let arch_name = arch.as_canonical_str();
        let image_url = format!("{base_url}AlmaLinux-{name}-GenericCloud-latest.{arch_name}.qcow2");
        let checksum_url = format!("{base_url}CHECKSUM");
        Some(ImageInfo {
            names: vec![name.to_string()],
            image_url,
            checksum_url,
            hash_alg: HashAlg::Sha256,
        })
    }
}
