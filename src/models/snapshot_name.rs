use crate::models::InstanceName;
use crate::util;
use std::fmt;
use std::str::FromStr;

/// A fully qualified snapshot reference, written as `<instance>/<snapshot>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SnapshotName {
    instance: InstanceName,
    name: String,
}

impl SnapshotName {
    pub fn get_instance(&self) -> &InstanceName {
        &self.instance
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl FromStr for SnapshotName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (instance, name) = value
            .split_once('/')
            .ok_or("Snapshot must be written as <instance>/<snapshot>")?;
        if !util::is_name(name) {
            return Err(
                "Snapshot name must only contain letters, numbers, underlines and dashes"
                    .to_string(),
            );
        }
        Ok(Self {
            instance: InstanceName::from_str(instance)?,
            name: name.to_string(),
        })
    }
}

impl fmt::Display for SnapshotName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.instance, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_name() {
        let snapshot = SnapshotName::from_str("mymachine/10clean-state_5").unwrap();

        assert_eq!(snapshot.get_instance().as_str(), "mymachine");
        assert_eq!(snapshot.as_str(), "10clean-state_5");
        assert_eq!(snapshot.to_string(), "mymachine/10clean-state_5");
    }

    #[test]
    fn test_reject_invalid_names() {
        assert!(SnapshotName::from_str("clean").is_err());
        assert!(SnapshotName::from_str("mymachine/..").is_err());
        assert!(SnapshotName::from_str("mymachine/a/b").is_err());
        assert!(SnapshotName::from_str("/clean").is_err());
        assert!(SnapshotName::from_str("").is_err());
    }
}
