use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct RockyLinuxImageProvider {}

impl ImageProvider for RockyLinuxImageProvider {
    fn get_vendor(&self) -> String {
        "rockylinux".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://dl.rockylinux.org/pub/rocky/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
    }

    fn get_image_dir_url(&self, name: &str, arch: Arch) -> String {
        let base_url = self.get_image_list_url();
        let arch_name = arch.as_canonical_str();
        format!("{base_url}{name}/images/{arch_name}/",)
    }

    fn get_image_info(&self, _content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        let base_url = self.get_image_dir_url(name, arch);
        let arch_name = arch.as_canonical_str();
        let image_url =
            format!("{base_url}Rocky-{name}-GenericCloud-Base.latest.{arch_name}.qcow2");
        let checksum_url = format!("{image_url}.CHECKSUM");
        Some(ImageInfo {
            names: vec![name.to_string()],
            image_url,
            checksum_url,
            hash_alg: HashAlg::Sha256,
        })
    }
}
