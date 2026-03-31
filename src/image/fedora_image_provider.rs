use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct FedoraImageProvider {}

impl FedoraImageProvider {
    fn get_version_from_content(&self, content: &str) -> Option<String> {
        util::find_and_extract(
            r#"href=\"Fedora-Cloud-Base-Generic-[0-9]+-(.*)\..+\.qcow2\""#,
            content,
        )
        .into_iter()
        .next()
    }
}

impl ImageProvider for FedoraImageProvider {
    fn get_vendor(&self) -> String {
        "fedora".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://download.fedoraproject.org/pub/fedora/linux/releases/".to_string()
    }

    fn get_image_names(&self, content: &str) -> Vec<String> {
        util::find_and_extract(r#">([0-9]+)/<"#, content)
            .into_iter()
            .filter(|version| version.parse::<u32>().map(|v| v > 40).unwrap_or_default())
            .collect()
    }

    fn get_image_dir_url(&self, name: &str, arch: Arch) -> String {
        let base_url = self.get_image_list_url();
        let arch_name = arch.as_canonical_str();
        format!("{base_url}{name}/Cloud/{arch_name}/images/",)
    }

    fn get_image_info(&self, content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        self.get_version_from_content(content).map(|version| {
            let base_url = self.get_image_dir_url(name, arch);
            let arch_name = arch.as_canonical_str();
            let image_url =
                format!("{base_url}Fedora-Cloud-Base-Generic-{name}-{version}.{arch_name}.qcow2");
            let checksum_url = format!("{base_url}Fedora-Cloud-{name}-{arch_name}-CHECKSUM.qcow2");
            ImageInfo {
                names: vec![name.to_string()],
                image_url,
                checksum_url,
                hash_alg: HashAlg::Sha256,
            }
        })
    }
}
