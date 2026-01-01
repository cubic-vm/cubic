use crate::error::Error;
use rand_core::OsRng;
use ssh_key::{Algorithm, LineEnding, private::PrivateKey};
use std::path::Path;

#[derive(Default)]
pub struct SshKeyGenerator;

impl SshKeyGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_key(&self, private_key_path: &Path) -> Result<(), Error> {
        PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
            .map_err(Error::Ssh)?
            .write_openssh_file(private_key_path, LineEnding::LF)
            .map(|_| ())
            .map_err(Error::Ssh)
    }

    pub fn generate_public_key(&self, private_key_path: &Path) -> Result<String, Error> {
        PrivateKey::read_openssh_file(private_key_path)
            .map_err(Error::Ssh)?
            .public_key()
            .to_openssh()
            .map_err(Error::Ssh)
    }
}
