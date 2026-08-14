use russh::keys::ssh_key::{HashAlg, PublicKey};

#[derive(Debug, PartialEq)]
pub enum KeyCheck {
    /// No key was pinned yet, so the offered key is trusted on first use
    Unknown,
    Match,
    Changed,
}

/// Compares the host key a guest offers against the key pinned in the
/// instance config. The key itself is stored by the instance store, this
/// type only decides what the comparison means.
#[derive(Default)]
pub struct HostKeyChecker;

impl HostKeyChecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check_key(&self, pinned: Option<&str>, offered: &str) -> KeyCheck {
        match pinned {
            None => KeyCheck::Unknown,
            Some(pinned) if pinned.trim() == offered.trim() => KeyCheck::Match,
            Some(_) => KeyCheck::Changed,
        }
    }

    /// Formats a key the way OpenSSH shows it, falling back to the raw key
    /// when it cannot be parsed.
    pub fn get_fingerprint(&self, key: &str) -> String {
        PublicKey::from_openssh(key.trim())
            .map(|key| key.fingerprint(HashAlg::Sha256).to_string())
            .unwrap_or_else(|_| key.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use getrandom::SysRng;
    use getrandom::rand_core::UnwrapErr;
    use russh::keys::ssh_key::{Algorithm, PrivateKey};

    fn build_key() -> String {
        PrivateKey::random(&mut UnwrapErr(SysRng), Algorithm::Ed25519)
            .unwrap()
            .public_key()
            .to_openssh()
            .unwrap()
    }

    #[test]
    fn test_check_key_without_pinned_key_is_unknown() {
        assert_eq!(
            HostKeyChecker::new().check_key(None, &build_key()),
            KeyCheck::Unknown
        );
    }

    #[test]
    fn test_check_key_with_same_key_matches() {
        let key = build_key();

        assert_eq!(
            HostKeyChecker::new().check_key(Some(&key), &format!("{key}\n")),
            KeyCheck::Match
        );
    }

    #[test]
    fn test_check_key_with_other_key_has_changed() {
        assert_eq!(
            HostKeyChecker::new().check_key(Some(&build_key()), &build_key()),
            KeyCheck::Changed
        );
    }

    #[test]
    fn test_get_fingerprint_of_key() {
        let fingerprint = HostKeyChecker::new().get_fingerprint(&build_key());

        assert!(fingerprint.starts_with("SHA256:"));
    }

    #[test]
    fn test_get_fingerprint_of_broken_key() {
        assert_eq!(HostKeyChecker::new().get_fingerprint("broken"), "broken");
    }
}
