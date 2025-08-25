use crate::image::Arch;
use regex::Regex;
use std::fmt;
use std::str::FromStr;

#[cfg(target_arch = "aarch64")]
fn get_default_arch() -> Arch {
    Arch::ARM64
}

#[cfg(not(target_arch = "aarch64"))]
fn get_default_arch() -> Arch {
    Arch::AMD64
}

#[derive(Clone)]
pub struct ImageName {
    vendor: String,
    name: String,
    arch: Arch,
}

impl ImageName {
    pub fn get_vendor(&self) -> &str {
        &self.vendor
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_arch(&self) -> Arch {
        self.arch
    }
}

impl FromStr for ImageName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if Regex::new("^(\\w+):(\\w+)(:(amd64|arm64))?$")
            .unwrap()
            .is_match(name)
        {
            let mut tokens = name.split(':');
            let vendor = tokens.next().unwrap().to_string();
            let name = tokens.next().unwrap().to_string();
            let arch = tokens
                .next()
                .map(|x| Arch::from_str(x).unwrap())
                .unwrap_or(get_default_arch());

            Ok(Self { vendor, name, arch })
        } else {
            Err(
                "Image name must have the format: vendor:name[:arch] (e.g. debain:bookworm, debian:buster:amd64)"
                    .to_string(),
            )
        }
    }
}

impl fmt::Display for ImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debain_bookworm() {
        let image = ImageName::from_str("debian:bookworm").unwrap();
        assert_eq!(image.get_vendor(), "debian");
        assert_eq!(image.get_name(), "bookworm");
    }

    #[test]
    fn test_debain_bookworm_amd64() {
        let image = ImageName::from_str("debian:buster:amd64").unwrap();
        assert_eq!(image.get_vendor(), "debian");
        assert_eq!(image.get_name(), "buster");
        assert_eq!(image.get_arch(), Arch::AMD64);
    }

    #[test]
    fn test_debain_bookworm_arm64() {
        let image = ImageName::from_str("debian:bookworm:arm64").unwrap();
        assert_eq!(image.get_vendor(), "debian");
        assert_eq!(image.get_name(), "bookworm");
        assert_eq!(image.get_arch(), Arch::ARM64);
    }
}
