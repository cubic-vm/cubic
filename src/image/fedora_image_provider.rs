use crate::arch::Arch;
use crate::image::{HashAlg, ImageProvider};
use crate::util;

pub struct FedoraImageProvider {}

impl ImageProvider for FedoraImageProvider {
    fn get_vendor(&self) -> &str {
        "fedora"
    }

    fn get_base_url(&self) -> &str {
        "https://download.fedoraproject.org/pub/fedora/linux/releases/"
    }

    fn find_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
            .into_iter()
            .filter(|version| version.parse::<u32>().map(|v| v > 40).unwrap_or_default())
            .collect()
    }

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("{name}/Cloud/{arch_name}/images/",)
    }

    fn get_image_names(&self, _image_file: &str, name: &str) -> Vec<String> {
        vec![name.to_string()]
    }

    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        format!("Fedora-Cloud-Base-Generic-{name}-.*.{arch_name}.qcow2")
    }

    fn get_checksum_file(&self, image_file: &str, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_canonical_str();
        image_file.replace("-Base-Generic", "").replace(
            &format!(".{arch_name}.qcow2"),
            &format!("-{arch_name}-CHECKSUM"),
        )
    }

    fn get_checksum_alg(&self) -> HashAlg {
        HashAlg::Sha256
    }
}
