use crate::error::{Error, Result};
use crate::models::InstanceName;
use clap::Parser;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct InstanceArg {
    /// Name of the virtual machine instance
    #[clap(id = "instance", value_name = "INSTANCE")]
    pub value: InstanceName,
}

impl From<InstanceName> for InstanceArg {
    fn from(name: InstanceName) -> Self {
        Self { value: name }
    }
}

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct InstancesArg {
    /// Names of the virtual machine instances
    #[clap(id = "instances", value_name = "INSTANCES")]
    pub value: Vec<InstanceName>,
}

impl InstancesArg {
    /// Reject an empty name list. This is a runtime check instead of a clap
    /// required constraint because `stop --all` accepts an empty list.
    pub fn require_names(&self) -> Result<()> {
        if self.value.is_empty() {
            Err(Error::MissingInstanceName)
        } else {
            Ok(())
        }
    }

    pub fn get_names(&self) -> Vec<String> {
        self.value.iter().map(ToString::to_string).collect()
    }
}

impl From<InstanceName> for InstancesArg {
    fn from(name: InstanceName) -> Self {
        Self { value: vec![name] }
    }
}

impl From<Vec<InstanceName>> for InstancesArg {
    fn from(names: Vec<InstanceName>) -> Self {
        Self { value: names }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_name() {
        assert!(InstanceArg::try_parse_from(["prog", "my-instance_1"]).is_ok());
    }

    #[test]
    fn test_reject_parent_traversal() {
        assert!(InstanceArg::try_parse_from(["prog", "../../etc"]).is_err());
    }

    #[test]
    fn test_reject_absolute_path() {
        assert!(InstanceArg::try_parse_from(["prog", "/abs/path"]).is_err());
    }

    #[test]
    fn test_parse_valid_names() {
        assert!(InstancesArg::try_parse_from(["prog", "trixie", "noble"]).is_ok());
    }

    #[test]
    fn test_reject_parent_traversal_in_list() {
        assert!(InstancesArg::try_parse_from(["prog", "trixie", "../../etc"]).is_err());
    }

    #[test]
    fn test_reject_absolute_path_in_list() {
        assert!(InstancesArg::try_parse_from(["prog", "/abs/path"]).is_err());
    }

    #[test]
    fn test_require_names_rejects_empty_list() {
        let args: InstancesArg = Vec::new().into();
        assert!(matches!(
            args.require_names(),
            Err(Error::MissingInstanceName)
        ));
    }

    #[test]
    fn test_require_names_accepts_names() {
        use std::str::FromStr;
        let args: InstancesArg = vec![InstanceName::from_str("foo").unwrap()].into();
        assert!(args.require_names().is_ok());
    }

    #[test]
    fn test_get_names() {
        use std::str::FromStr;
        let args: InstancesArg = vec![
            InstanceName::from_str("foo").unwrap(),
            InstanceName::from_str("bar").unwrap(),
        ]
        .into();
        assert_eq!(args.get_names(), vec!["foo", "bar"]);
    }
}
