use crate::arch::Arch;
use crate::image::HashAlg;

pub trait ImageProvider {
    fn get_vendor(&self) -> &str;

    fn get_base_url(&self) -> &str;
    fn find_image_names(&self, content: &str) -> Vec<String>;

    fn get_image_dir_path(&self, name: &str, arch: Arch) -> String;
    fn get_image_names(&self, image_file: &str, name: &str) -> Vec<String>;
    fn get_image_file_pattern(&self, name: &str, arch: Arch) -> String;

    fn get_checksum_file(&self, image_file: &str, name: &str, arch: Arch) -> String;
    fn get_checksum_alg(&self) -> HashAlg;
}
