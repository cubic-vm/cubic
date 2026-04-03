use crate::arch::Arch;
use crate::image::{HashAlg, ImageProvider};
use crate::util;

pub struct AlmaLinuxImageProvider {}

impl ImageProvider for AlmaLinuxImageProvider {
    fn get_vendor(&self) -> &str {
        "almalinux"
    }

    fn get_base_url(&self) -> &str {
        "https://raw.repo.almalinux.org/almalinux/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}/cloud/{arch_name}/images/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("AlmaLinux-{name}-GenericCloud-latest.{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, _image_file: &str, _name: &str, _arch: Arch) -> String {
        "CHECKSUM".to_string()
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }
}
