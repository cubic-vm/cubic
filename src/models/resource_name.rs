use crate::models::{ImageName, InstanceName, SnapshotName};
use std::fmt;
use std::str::FromStr;

/// A VM instance, snapshot or image, written as `<instance>`,
/// `<instance>/<snapshot>` or `<distro>:<tag>`. A plain name is an instance.
#[derive(Clone, Debug)]
pub enum ResourceName {
    Instance(InstanceName),
    Snapshot(SnapshotName),
    Image(ImageName),
}

type Parser = fn(&str) -> Result<ResourceName, String>;

impl FromStr for ResourceName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let parsers: [Parser; 3] = [
            |name: &str| name.parse().map(Self::Instance),
            |name: &str| name.parse().map(Self::Snapshot),
            |name: &str| name.parse().map(Self::Image),
        ];

        let mut errors = Vec::new();
        for parse in parsers {
            match parse(name) {
                Ok(resource) => return Ok(resource),
                Err(error) => errors.push(format!("   - {error}")),
            }
        }
        Err(format!("expected one of\n{}", errors.join("\n")))
    }
}

impl fmt::Display for ResourceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Instance(instance) => instance.fmt(f),
            Self::Snapshot(snapshot) => snapshot.fmt(f),
            Self::Image(image) => image.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_name_is_an_instance() {
        let resource = ResourceName::from_str("debian").unwrap();
        assert!(matches!(resource, ResourceName::Instance(_)));
    }

    #[test]
    fn test_lists_every_format_when_nothing_parses() {
        let error = ResourceName::from_str("a b").unwrap_err();
        let formats = error.lines().filter(|line| line.starts_with("   - "));
        assert_eq!(formats.count(), 3);
    }
}
