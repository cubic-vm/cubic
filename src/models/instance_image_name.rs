use crate::models::{ImageName, InstanceName};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum InstanceImageName {
    Image(ImageName),
    Instance(InstanceName),
}

impl FromStr for InstanceImageName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let image = ImageName::from_str(name);
        let instance = InstanceName::from_str(name);

        match (image, instance) {
            (Ok(i), _) => Ok(InstanceImageName::Image(i)),
            (_, Ok(i)) => Ok(InstanceImageName::Instance(i)),
            (Err(a), Err(b)) => Err(format!("{a}\n{b}")),
        }
    }
}

impl fmt::Display for InstanceImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            InstanceImageName::Image(image) => image.fmt(f),
            InstanceImageName::Instance(instance) => instance.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_name_takes_precedence() {
        assert!(matches!(
            InstanceImageName::from_str("debian:bookworm").unwrap(),
            InstanceImageName::Image(_)
        ));
    }

    #[test]
    fn test_plain_name_parses_as_instance() {
        assert!(matches!(
            InstanceImageName::from_str("mymachine").unwrap(),
            InstanceImageName::Instance(_)
        ));
    }

    #[test]
    fn test_reject_name_combines_both_errors() {
        let error = InstanceImageName::from_str("foo/bar").unwrap_err();
        assert!(error.contains("Image name"));
        assert!(error.contains("Instance name"));
    }

    #[test]
    fn test_to_string() {
        assert_eq!(
            InstanceImageName::from_str("debian:bookworm:arm64")
                .unwrap()
                .to_string(),
            "debian:bookworm:arm64"
        );
        assert_eq!(
            InstanceImageName::from_str("mymachine")
                .unwrap()
                .to_string(),
            "mymachine"
        );
    }
}
