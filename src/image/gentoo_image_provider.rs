use crate::arch::Arch;
use crate::image::{HashAlg, ImageInfo, ImageProvider};
use crate::util;

pub struct GentooImageProvider {}

impl GentooImageProvider {
    fn get_timestamp(&self, content: &str) -> Option<String> {
        util::find_and_extract(
            r#"href=\"di-[A-Za-z0-9]+-cloudinit-([A-Za-z0-9]+).qcow2\""#,
            content,
        )
        .into_iter()
        .next()
    }
}

impl ImageProvider for GentooImageProvider {
    fn get_vendor(&self) -> String {
        "gentoo".to_string()
    }

    fn get_image_list_url(&self) -> String {
        "https://distfiles.gentoo.org/releases/".to_string()
    }

    fn get_image_names(&self, _content: &str) -> Vec<String> {
        vec!["latest".to_string()]
    }

    fn get_image_dir_url(&self, _name: &str, arch: Arch) -> String {
        let arch_name = arch.as_vendor_str();
        format!(
            "{}/{arch_name}/autobuilds/current-di-{arch_name}-cloudinit/",
            self.get_image_list_url()
        )
    }

    fn get_image_info(&self, content: &str, name: &str, arch: Arch) -> Option<ImageInfo> {
        self.get_timestamp(content).map(|timestamp| {
            let base_url = self.get_image_dir_url(name, arch);
            let arch_name = arch.as_vendor_str();
            let image_url = format!("{base_url}di-{arch_name}-cloudinit-{timestamp}.qcow2");
            let checksum_url = format!("{image_url}.sha256");
            ImageInfo {
                names: vec![name.to_string()],
                image_url,
                checksum_url,
                hash_alg: HashAlg::Sha256,
            }
        })
    }
}
