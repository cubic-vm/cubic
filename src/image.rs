pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;
pub mod image_name;
pub mod image_store;
pub mod image_store_mock;

use crate::arch::Arch;
use crate::error::Error;
pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;
pub use image_name::*;
pub use image_store::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Image {
    pub vendor: String,
    pub names: Vec<String>,
    pub arch: Arch,
    pub url: String,
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

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct ImageCache {
    images: Vec<Image>,
    timestamp: u64,
}

impl ImageCache {
    pub fn deserialize(reader: &mut dyn Read) -> Result<ImageCache, Error> {
        serde_yaml::from_reader(reader).map_err(Error::SerdeYaml)
    }

    pub fn serialize(&self, writer: &mut dyn Write) -> Result<(), Error> {
        serde_yaml::to_writer(writer, self).map_err(Error::SerdeYaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::BufReader;

    #[test]
    fn test_deserialize_invalid() {
        let reader = &mut BufReader::new("".as_bytes());
        let cache = ImageCache::deserialize(reader);
        assert!(cache.is_err());
    }

    #[test]
    fn test_deserialize_empty() {
        let reader = &mut BufReader::new("images: []\ntimestamp: 0".as_bytes());
        let cache = ImageCache::deserialize(reader);
        assert_eq!(cache.unwrap(), ImageCache::default());
    }

    #[test]
    fn test_serialize_empty() {
        let mut writer = Vec::new();

        ImageCache::default().serialize(&mut writer).unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "images: []\ntimestamp: 0\n"
        );
    }

    #[test]
    fn test_deserialize_single() {
        let reader = &mut BufReader::new("images: []\ntimestamp: 0".as_bytes());
        let cache = ImageCache::deserialize(reader);
        assert_eq!(cache.unwrap(), ImageCache::default());
    }

    #[test]
    fn test_serialize_single() {
        let mut writer = Vec::new();

        ImageCache {
            images: vec![Image {
                vendor: "testvendor".to_string(),
                names: vec!["testversion".to_string(), "testcodename".to_string()],
                arch: Arch::AMD64,
                url: "testurl".to_string(),
                size: None,
            }],
            timestamp: 1000,
        }
        .serialize(&mut writer)
        .unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            r#"images:
- vendor: testvendor
  names:
  - testversion
  - testcodename
  arch: AMD64
  url: testurl
  size: null
timestamp: 1000
"#
        );
    }
}
