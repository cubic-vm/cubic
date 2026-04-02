mod almalinux_image_provider;
mod archlinux_image_provider;
mod debian_image_provider;
mod fedora_image_provider;
mod gentoo_image_provider;
mod image_cache;
mod image_dao;
mod image_factory;
mod image_fetcher;
mod image_name;
mod image_provider;
mod image_store;
mod image_store_mock;
mod opensuse_image_provider;
mod rockylinux_image_provider;
mod ubuntu_image_provider;

use crate::arch::Arch;
pub use almalinux_image_provider::*;
pub use archlinux_image_provider::*;
pub use debian_image_provider::*;
pub use fedora_image_provider::*;
pub use gentoo_image_provider::*;
pub use image_cache::*;
pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;
pub use image_name::*;
pub use image_provider::*;
pub use image_store::*;
#[cfg(test)]
pub use image_store_mock::tests::ImageStoreMock;
pub use opensuse_image_provider::*;
pub use rockylinux_image_provider::*;
use serde::{Deserialize, Serialize};
pub use ubuntu_image_provider::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlg {
    Sha512,
    Sha256,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Image {
    pub vendor: String,
    pub names: Vec<String>,
    pub arch: Arch,
    pub image_url: String,
    pub checksum_url: String,
    pub hash_alg: HashAlg,
    pub size: Option<u64>,
}

impl Image {
    pub fn get_version(&self) -> &str {
        &self.names[0]
    }

    pub fn get_name(&self) -> &str {
        if self.names.len() > 1 {
            &self.names[1]
        } else {
            &self.names[0]
        }
    }

    pub fn get_image_names(&self) -> String {
        format!(
            "{}:{}",
            self.vendor,
            if self.names.len() > 1 {
                format!("{{{}}}", self.names.join(", "))
            } else {
                self.names[0].clone()
            }
        )
    }

    pub fn to_name(&self) -> String {
        format!("{}:{}:{}", self.vendor, self.get_version(), self.arch)
    }

    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.vendor, self.get_name(), self.arch)
    }
}

use std::cmp::{Ord, Ordering};

impl Ord for Image {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut result = self.vendor.cmp(&other.vendor);

        if result == Ordering::Equal {
            let a = &self.names[0];
            let b = &other.names[0];

            if let Ok(a) = a.parse::<u32>()
                && let Ok(b) = b.parse::<u32>()
            {
                result = a.cmp(&b);
            } else {
                result = a.cmp(b);
            }
        }

        result
    }
}

impl PartialOrd for Image {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
