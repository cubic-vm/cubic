use crate::error::Error;
use crate::instance::{InstanceStore, Target, TargetInstancePath};
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct TargetPath {
    target: Option<Target>,
    path: String,
}

impl TargetPath {
    pub fn get_target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn to_target_instance_path(
        &self,
        instance_store: &dyn InstanceStore,
    ) -> Result<TargetInstancePath, Error> {
        if let Some(target) = self.target.as_ref() {
            let instance = instance_store.load(target.get_instance().as_str())?;
            Ok(TargetInstancePath {
                user: target.get_user().cloned(),
                instance: Some(instance),
                path: self.path.clone(),
            })
        } else {
            Ok(TargetInstancePath {
                user: None,
                instance: None,
                path: self.path.clone(),
            })
        }
    }
}

impl FromStr for TargetPath {
    type Err = String;

    fn from_str(target_path: &str) -> Result<Self, Self::Err> {
        match *target_path.split(':').collect::<Vec<_>>().as_slice() {
            [path] => Ok(Self {
                target: None,
                path: path.to_string(),
            }),

            [target, path] => Ok(Self {
                target: Some(Target::from_str(target)?),
                path: path.to_string(),
            }),
            _ => Err("Target path must have the format [[user@]instance:]path (e.g. 'cubic@mymachine:/home/cubic', 'mymachine:~/' or 'my_file'".to_string()),
        }
    }
}

impl fmt::Display for TargetPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut result = Ok(());

        if let Some(target) = self.target.as_ref() {
            result = write!(f, "{target}:")
        }
        result.or(write!(f, "{}", self.path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let path = TargetPath::from_str("/home/cubic").unwrap();
        assert_eq!(path.to_string().as_str(), "/home/cubic");
    }

    #[test]
    fn test_instance_path() {
        let path = TargetPath::from_str("mymachine:/home/cubic").unwrap();
        assert_eq!(path.to_string().as_str(), "mymachine:/home/cubic");
    }

    #[test]
    fn test_user_instance_path() {
        let path = TargetPath::from_str("cubic@mymachine:/home/cubic").unwrap();
        assert_eq!(path.to_string().as_str(), "cubic@mymachine:/home/cubic");
    }
}
