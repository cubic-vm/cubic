use crate::instance::Instance;
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct TargetInstancePath {
    pub user: Option<String>,
    pub instance: Option<Instance>,
    pub path: String,
}

impl TargetInstancePath {
    pub fn to_pathbuf(&self) -> PathBuf {
        let re = Regex::new("^~").unwrap();
        let path = if let Some(user) = self
            .user
            .clone()
            .or(self.instance.as_ref().map(|i| i.user.clone()))
        {
            re.replace_all(&self.path, &format!("/home/{user}"))
                .to_string()
        } else {
            self.path.to_string()
        };

        Path::new(&path).to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        instance.user = "root".to_string();
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
        instance.user = "root".to_string();
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
