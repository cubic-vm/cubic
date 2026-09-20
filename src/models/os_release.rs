use crate::models::Image;

/// Guest OS as reported by /etc/os-release
pub struct OsRelease;

impl OsRelease {
    /// Joins the ID and VERSION_ID fields into a name such as ubuntu:24.04. A
    /// rolling release ships no version, so it becomes arch:rolling. Returns
    /// nothing without an ID, because the name builds on it.
    pub fn parse_name(content: &str) -> Option<String> {
        let mut id = None;
        let mut version_id = None;

        for line in content.lines() {
            match line.trim().split_once('=') {
                Some(("ID", value)) => id = Self::unquote(value),
                Some(("VERSION_ID", value)) => version_id = Self::unquote(value),
                _ => (),
            }
        }

        id.map(|id| format!("{id}:{}", version_id.unwrap_or(Image::ROLLING)))
    }

    fn unquote(value: &str) -> Option<&str> {
        let value = value.trim().trim_matches('"').trim_matches('\'');

        (!value.is_empty()).then_some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_a_quoted_release() {
        let content = r#"
PRETTY_NAME="Ubuntu 24.04.1 LTS"
NAME="Ubuntu"
ID=ubuntu
ID_LIKE=debian
VERSION_ID="24.04"
"#;

        assert_eq!(OsRelease::parse_name(content).unwrap(), "ubuntu:24.04");
    }

    #[test]
    fn test_parse_a_rolling_release_without_a_version() {
        let content = "NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\n";

        assert_eq!(OsRelease::parse_name(content).unwrap(), "arch:rolling");
    }

    #[test]
    fn test_parse_ignores_comments_and_empty_values() {
        let content = "# a comment\n# ID=fake\n\nID='fedora'\nVERSION_ID=\"\"\n";

        assert_eq!(OsRelease::parse_name(content).unwrap(), "fedora:rolling");
    }

    #[test]
    fn test_parse_needs_an_id() {
        assert!(OsRelease::parse_name("").is_none());
        assert!(OsRelease::parse_name("VERSION_ID=\"24.04\"").is_none());
        assert!(OsRelease::parse_name("ID=").is_none());
    }
}
