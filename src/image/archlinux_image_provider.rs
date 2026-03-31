use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};

pub struct ArchLinuxImageProvider {}

impl ImageProvider for ArchLinuxImageProvider {
    fn get_vendor(&self) -> String {
        "archlinux".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://geo.mirror.pkgbuild.com/images/".to_string()
    }

    fn get_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn get_image_dir_url(&self, _name: &str, _arch: Arch) -> String {
        format!("{}latest/", self.get_image_list_url())
    }

    fn get_image_info(&self, _content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        let base_url = self.get_image_dir_url(name, arch);
        let arch_name = arch.as_canonical_str();
        let image_url = format!("{base_url}Arch-Linux-{arch_name}-cloudimg.qcow2");
        let checksum_url = format!("{image_url}.SHA256");
        Some(ImageInfo {
            names: vec![name.to_string()],
            image_url,
            checksum_url,
            hash_alg: HashAlg::Sha256,
        })
    }
}
