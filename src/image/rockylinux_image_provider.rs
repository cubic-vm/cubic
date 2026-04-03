use crate::arch::Arch;
use crate::image::{HashAlg, ImageProvider};
use crate::util;

pub struct RockyLinuxImageProvider {}

impl ImageProvider for RockyLinuxImageProvider {
    fn get_vendor(&self) -> &str {
        "rockylinux"
    }

    fn get_base_url(&self) -> &str {
        "https://dl.rockylinux.org/pub/rocky/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}/images/{arch_name}/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("Rocky-{name}-GenericCloud-Base.latest.{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.CHECKSUM")
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }
}
