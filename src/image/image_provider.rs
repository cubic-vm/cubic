use crate::arch::Arch;
use crate::image::HashAlg;

pub struct ImageInfo {
    pub names: Vec<String>,
    pub image_url: String,
    pub checksum_url: String,
    pub hash_alg: HashAlg,
}

pub trait ImageProvider {
    fn get_vendor(&self) -> String;

    fn get_image_list_url(&self) -> String;
    fn get_image_names(&self, content: &str) -> Vec<String>;

    fn get_image_dir_url(&self, name: &str, arch: Arch) -> String;
    fn get_image_info(&self, content: &str, name: &str, arch: Arch) -> Option<ImageInfo>;
}
