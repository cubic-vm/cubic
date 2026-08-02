use crate::models::{Environment, Image};
use crate::platform::System;
use std::path::Path;

#[derive(Default)]
pub struct ImageStore;

impl ImageStore {
    pub fn new() -> Self {
        ImageStore
    }

    pub fn exists(&self, system: &dyn System, env: &Environment, image: &Image) -> bool {
        system.exists_path(Path::new(&env.get_image_file(&image.to_file_name())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Arch, HashAlg, UserName};
    use crate::platform::SystemMock;
    use std::str::FromStr;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            "/cache".to_string(),
            String::new(),
        )
    }

    fn build_image(arch: Arch) -> Image {
        Image {
            vendor: "debian".to_string(),
            names: vec!["12".to_string(), "bookworm".to_string()],
            arch,
            image_url: String::new(),
            checksum_url: String::new(),
            hash_alg: HashAlg::Sha512,
            size: None,
        }
    }

    #[test]
    fn test_exists_finds_a_downloaded_image() {
        let system = SystemMock::new().add_file("/cache/images/debian_bookworm_amd64", b"");

        assert!(ImageStore::new().exists(&system, &build_env(), &build_image(Arch::AMD64)));
    }

    #[test]
    fn test_exists_tells_the_architectures_apart() {
        let system = SystemMock::new().add_file("/cache/images/debian_bookworm_amd64", b"");

        assert!(!ImageStore::new().exists(&system, &build_env(), &build_image(Arch::ARM64)));
    }
}
