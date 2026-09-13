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

    fn build_path(user: Option<&str>, instance_user: Option<&str>, path: &str) -> String {
        TargetInstancePath {
            user: user.map(str::to_string),
            instance: instance_user.map(|name| Instance {
                user: UserName::from_str(name).unwrap(),
                ..Instance::default()
            }),
            path: path.to_string(),
        }
        .to_pathbuf()
        .to_str()
        .unwrap()
        .to_string()
    }

    #[test]
    fn test_expand_a_tilde() {
        assert_eq!(build_path(None, None, "a/b/c"), "a/b/c");
        assert_eq!(build_path(None, None, "~/a/b/c"), "~/a/b/c");
        assert_eq!(build_path(Some("tux"), None, "~"), "/home/tux");
        assert_eq!(build_path(Some("tux"), None, "~/a/b/c"), "/home/tux/a/b/c");
        assert_eq!(
            build_path(None, Some("root"), "~/a/b/c"),
            "/home/root/a/b/c"
        );

        // The name given on the command line wins over the instance one.
        let path = build_path(Some("tux"), Some("root"), "~/a/b/c");
        assert_eq!(path, "/home/tux/a/b/c");
    }
}
