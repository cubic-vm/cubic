use crate::error::Error;
use crate::image::Image;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const IMAGE_CACHE_LIFETIME_SEC: u64 = 7 * 24 * 60 * 60; // = 1 week

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct ImageCache {
    pub images: Vec<Image>,
    timestamp: u64,
}

impl ImageCache {
    pub fn new(images: Vec<Image>) -> Self {
        ImageCache {
            images: images.to_vec(),
            timestamp: Self::get_timestamp(),
        }
    }

    pub fn is_valid(&self) -> bool {
        (Self::get_timestamp() - self.timestamp) < IMAGE_CACHE_LIFETIME_SEC
    }

    pub fn read_from_file(path: &str) -> Option<Self> {
        File::open(path)
            .map_err(Error::Io)
            .and_then(|ref mut reader| ImageCache::deserialize(reader))
            .ok()
    }

    pub fn write_to_file(&self, path: &str) {
        File::create(path)
            .map_err(Error::Io)
            .and_then(|mut file| self.serialize(&mut file))
            .ok();
    }

    fn deserialize(reader: &mut dyn Read) -> Result<ImageCache, Error> {
        let mut data = String::new();
        reader.read_to_string(&mut data).map_err(Error::Io)?;
        toml::from_str(&data).map_err(|_| Error::CannotParseFile(String::new()))
    }

    fn serialize(&self, writer: &mut dyn Write) -> Result<(), Error> {
        toml::to_string(self)
            .map(|content| writer.write_all(&content.into_bytes()))
            .map(|_| ())
            .map_err(Error::SerdeToml)
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|time| time.as_secs())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::image::{Arch, HashAlg};
    use std::io::BufReader;

    #[test]
    fn test_deserialize_invalid() {
        let reader = &mut BufReader::new("".as_bytes());
        let cache = ImageCache::deserialize(reader);
        assert!(cache.is_err());
    }

    #[test]
    fn test_deserialize_empty() {
        let reader = &mut BufReader::new("images = []\ntimestamp = 0".as_bytes());
        let cache = ImageCache::deserialize(reader);
        assert_eq!(cache.unwrap(), ImageCache::default());
    }

    #[test]
    fn test_serialize_empty() {
        let mut writer = Vec::new();

        ImageCache::default().serialize(&mut writer).unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "images = []\ntimestamp = 0\n"
        );
    }

    #[test]
    fn test_deserialize_single() {
        let reader = &mut BufReader::new(
            r#"timestamp = 5000

[[images]]
vendor = "testvendor"
names = ["testversion", "testcodename"]
arch = "AMD64"
image_url = "imageurl"
checksum_url = "checksumurl"
hash_alg = "Sha256"

[[images]]
vendor = "testvendor2"
names = ["testversion2", "testcodename2"]
arch = "ARM64"
image_url = "imageurl2"
checksum_url = "checksumurl2"
hash_alg = "Sha512"

"#
            .as_bytes(),
        );
        let cache = ImageCache::deserialize(reader).unwrap();
        assert_eq!(cache.timestamp, 5000);
        assert_eq!(cache.images.len(), 2);
        assert_eq!(cache.images[0].vendor, "testvendor");
        assert_eq!(cache.images[0].names, ["testversion", "testcodename"]);
        assert_eq!(cache.images[0].arch, Arch::AMD64);
        assert_eq!(cache.images[0].image_url, "imageurl");
        assert_eq!(cache.images[0].checksum_url, "checksumurl");
        assert_eq!(cache.images[0].hash_alg, HashAlg::Sha256);
        assert_eq!(cache.images[1].vendor, "testvendor2");
        assert_eq!(cache.images[1].names, ["testversion2", "testcodename2"]);
        assert_eq!(cache.images[1].arch, Arch::ARM64);
        assert_eq!(cache.images[1].image_url, "imageurl2");
        assert_eq!(cache.images[1].checksum_url, "checksumurl2");
        assert_eq!(cache.images[1].hash_alg, HashAlg::Sha512);
    }

    #[test]
    fn test_serialize_single() {
        let mut writer = Vec::new();

        ImageCache {
            images: vec![Image {
                vendor: "testvendor".to_string(),
                names: vec!["testversion".to_string(), "testcodename".to_string()],
                arch: Arch::AMD64,
                image_url: "imageurl".to_string(),
                checksum_url: "checksumurl".to_string(),
                hash_alg: HashAlg::Sha256,
                size: None,
            }],
            timestamp: 1000,
        }
        .serialize(&mut writer)
        .unwrap();

        assert_eq!(
            String::from_utf8(writer).unwrap(),
            r#"timestamp = 1000

[[images]]
vendor = "testvendor"
names = ["testversion", "testcodename"]
arch = "AMD64"
image_url = "imageurl"
checksum_url = "checksumurl"
hash_alg = "Sha256"
"#
        );
    }
}
