use crate::arch::Arch;
use crate::image::{HashAlg, ImageProvider};
use crate::util;

pub struct OpenSuseImageProvider {}

impl ImageProvider for OpenSuseImageProvider {
    fn get_vendor(&self) -> &str {
        "opensuse"
    }

    fn get_base_url(&self) -> &str {
        "https://download.opensuse.org/repositories/Cloud:/Images:/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">Leap_([0-9]+\.[0-9]+)/<"#, content)
    }

    fn get_image_dir_path(&self, name: &str, _arch: Arch) -> String {
        format!("Leap_{name}/images/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("openSUSE-Leap-{name}.{arch_name}-NoCloud.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, _arch: Arch) -> String {
        format!("{image_file}.sha256")
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }
}
