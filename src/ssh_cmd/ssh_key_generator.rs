use crate::error::{Error, Result};
use crate::platform::System;
use getrandom::SysRng;
use getrandom::rand_core::UnwrapErr;
use russh::keys::ssh_key::{Algorithm, LineEnding, PrivateKey};
use std::path::Path;

#[derive(Default)]
pub struct SshKeyGenerator;

impl SshKeyGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_key(&self, system: &dyn System, private_key_path: &Path) -> Result<()> {
        let key = PrivateKey::random(&mut UnwrapErr(SysRng), Algorithm::Ed25519)
            .map_err(Error::from)?
            .to_openssh(LineEnding::LF)
            .map_err(Error::from)?;

        system.write_secret_file(private_key_path, key.as_bytes())
    }

    pub fn generate_public_key(
        &self,
        system: &dyn System,
        private_key_path: &Path,
    ) -> Result<String> {
        let key = system.read_file_to_string(private_key_path)?;

        PrivateKey::from_openssh(&key)
            .map_err(Error::from)?
            .public_key()
            .to_openssh()
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_generate_key_then_generate_public_key_round_trips() {
        let system = SystemMock::new();
        let generator = SshKeyGenerator::new();
        let path = Path::new("/data/machines/test/ssh_client_key");

        generator.generate_key(&system, path).unwrap();
        let pubkey = generator.generate_public_key(&system, path).unwrap();

        assert!(pubkey.starts_with("ssh-ed25519 "));
    }

    #[test]
    fn test_generate_public_key_fails_when_key_missing() {
        let system = SystemMock::new();

        assert!(
            SshKeyGenerator::new()
                .generate_public_key(&system, Path::new("/data/missing"))
                .is_err()
        );
    }
}
