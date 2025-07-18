pub mod image_dao;
pub mod image_factory;
pub mod image_fetcher;

use crate::arch::Arch;
use crate::error::Error;
pub use image_dao::*;
pub use image_factory::*;
pub use image_fetcher::*;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Image {
    pub vendor: String,
    pub codename: String,
    pub version: String,
    pub arch: Arch,
    pub url: String,
    pub size: Option<u64>,
}

impl Image {
    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.vendor, self.codename, self.arch)
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
                codename: "testcodename".to_string(),
                version: "testversion".to_string(),
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
  codename: testcodename
  version: testversion
  arch: AMD64
  url: testurl
  size: null
timestamp: 1000
"#
        );
    }
}
