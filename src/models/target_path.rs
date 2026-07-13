use crate::models::Target;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct TargetPath {
    target: Option<Target>,
    pub(crate) path: String,
}

impl TargetPath {
    pub fn get_target(&self) -> Option<&Target> {
        self.target.as_ref()
    }
}

impl FromStr for TargetPath {
    type Err = String;

    fn from_str(target_path: &str) -> std::result::Result<Self, Self::Err> {
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
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

    #[test]
    fn test_reject_too_many_colons() {
        assert!(TargetPath::from_str("a:b:c").is_err());
    }

    #[test]
    fn test_get_target_of_instance_path() {
        let path = TargetPath::from_str("mymachine:/home/cubic").unwrap();
        assert_eq!(
            path.get_target().unwrap().get_instance().as_str(),
            "mymachine"
        );
    }

    #[test]
    fn test_get_target_of_local_path() {
        let path = TargetPath::from_str("/home/cubic").unwrap();
        assert!(path.get_target().is_none());
    }
}
