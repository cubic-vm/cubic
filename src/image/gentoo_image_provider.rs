use crate::arch::Arch;
use crate::image::{HashAlg, ImageProvider};

pub struct GentooImageProvider {}

impl ImageProvider for GentooImageProvider {
    fn get_vendor(&self) -> &str {
        "gentoo"
    }

    fn get_base_url(&self) -> &str {
        "https://distfiles.gentoo.org/releases/"
    }

    fn find_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn get_image_dir_path(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!("{arch_name}/autobuilds/current-di-{arch_name}-cloudinit/")
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!("di-{arch_name}-cloudinit-[A-Za-z0-9]+.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.sha256")
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }
}
