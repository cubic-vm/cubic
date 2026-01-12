pub mod image_cache;
pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;
pub mod image_name;
pub mod image_store;
pub mod image_store_mock;

use crate::arch::Arch;
pub use image_cache::*;
pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;
pub use image_name::*;
pub use image_store::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum HashAlg {
    Sha512,
    Sha256,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
