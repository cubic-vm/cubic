use crate::instance::InstanceName;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub struct Target {
    user: Option<String>,
    instance: InstanceName,
}

impl Target {
    pub fn from_instance_name(instance: InstanceName) -> Self {
        Self {
            user: None,
            instance,
        }
    }

    pub fn get_user(&self) -> Option<&String> {
        self.user.as_ref()
    }

    pub fn get_instance(&self) -> &InstanceName {
        &self.instance
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(target: &str) -> Result<Self, Self::Err> {
        match *target.split('@').collect::<Vec<_>>().as_slice() {
            [instance] => Ok(Self {
                user: None,
                instance: InstanceName::from_str(instance)?,
            }),

            [user, instance] => Ok(Self {
                user: Some(user.to_string()),
                instance: InstanceName::from_str(instance)?,
            }),
            _ => Err("Target name must have format 'user@instance' or 'instance'".to_string()),
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mut result = Ok(());

        if let Some(user) = self.user.as_ref() {
            result = write!(f, "{user}@")
        }
        result.or(write!(f, "{}", self.instance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_name() {
        let target = Target::from_str("mymachine").unwrap();
        assert_eq!(target.get_user(), None);
        assert_eq!(target.get_instance().as_str(), "mymachine");
    }

    #[test]
    fn test_user_and_instance_name() {
        let target = Target::from_str("cubic@mymachine").unwrap();
        assert_eq!(target.get_user().unwrap().as_str(), "cubic");
        assert_eq!(target.get_instance().as_str(), "mymachine");
    }

    #[test]
    fn test_invalid_instance_name() {
        assert!(Target::from_str("cubic@my&machine").is_err());
    }

    #[test]
    fn test_invalid_target() {
        assert!(Target::from_str("cubic@my@machine").is_err());
    }
}
