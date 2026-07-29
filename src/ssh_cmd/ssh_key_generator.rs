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
