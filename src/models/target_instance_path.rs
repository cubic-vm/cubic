use crate::models::Instance;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TargetInstancePath {
    pub user: Option<String>,
    pub instance: Option<Instance>,
    pub path: String,
}

impl TargetInstancePath {
    pub fn to_pathbuf(&self) -> PathBuf {
        let user = self
            .user
            .as_deref()
            .or(self.instance.as_ref().map(|i| i.user.as_str()));

        match (user, self.path.strip_prefix('~')) {
            (Some(user), Some(rest)) => PathBuf::from(format!("/home/{user}{rest}")),
            _ => PathBuf::from(&self.path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserName;
    use std::str::FromStr;

    #[test]
    fn test_to_pathbuf() {
        assert_eq!(
            TargetInstancePath {
                user: None,
                instance: None,
                path: "a/b/c".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "a/b/c"
        )
    }

    #[test]
    fn test_to_pathbuf_with_tilde() {
        assert_eq!(
            TargetInstancePath {
                user: None,
                instance: None,
                path: "~/a/b/c".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "~/a/b/c"
        )
    }

    #[test]
    fn test_to_pathbuf_tilde_only() {
        assert_eq!(
            TargetInstancePath {
                user: Some("tux".to_string()),
                instance: None,
                path: "~".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "/home/tux"
        )
    }

    #[test]
    fn test_to_pathbuf_tilde_without_instance() {
        assert_eq!(
            TargetInstancePath {
                user: Some("tux".to_string()),
                instance: None,
                path: "~/a/b/c".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "/home/tux/a/b/c"
        )
    }

    #[test]
    fn test_to_pathbuf_tilde_with_user_and_instance() {
        let mut instance = Instance::default();
        instance.user = UserName::from_str("root").unwrap();
        assert_eq!(
            TargetInstancePath {
                user: Some("tux".to_string()),
                instance: Some(instance),
                path: "~/a/b/c".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "/home/tux/a/b/c"
        )
    }

    #[test]
    fn test_to_pathbuf_tilde_with_instance_without_user() {
        let mut instance = Instance::default();
        instance.user = UserName::from_str("root").unwrap();
        assert_eq!(
            TargetInstancePath {
                user: None,
                instance: Some(instance),
                path: "~/a/b/c".to_string(),
            }
            .to_pathbuf()
            .to_str()
            .unwrap(),
            "/home/root/a/b/c"
        )
    }
}
