use crate::error::Result;
use crate::models::InstanceCertPaths;
use crate::platform::System;
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType, IsCa, KeyPair,
    KeyUsagePurpose,
};
use std::path::PathBuf;

pub struct InstanceCertGenerator<'a> {
    system: &'a dyn System,
    dir: PathBuf,
}

impl<'a> InstanceCertGenerator<'a> {
    pub fn new(system: &'a dyn System, dir: PathBuf) -> Self {
        Self { system, dir }
    }

    pub fn exists(&self) -> bool {
        self.system
            .exists_path(&InstanceCertPaths::load(&self.dir).ca_cert)
    }

    pub fn generate(&self) -> Result<InstanceCertPaths> {
        let certs = InstanceCertPaths::load(&self.dir);

        let ca_key = KeyPair::generate()?;
        let mut ca_params = CertificateParams::default();

        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, "Cubic CA");
            dn
        };
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca = CertifiedIssuer::self_signed(ca_params, ca_key)?;

        let server_key = KeyPair::generate()?;

        let server_cert =
            CertificateParams::new(vec!["localhost".to_string()])?.signed_by(&server_key, &ca)?;

        let client_key = KeyPair::generate()?;

        let client_cert = CertificateParams::default().signed_by(&client_key, &ca)?;

        self.system
            .write_file(&certs.ca_cert, ca.pem().as_bytes())?;
        self.system
            .write_file(&certs.server_cert, server_cert.pem().as_bytes())?;
        self.system
            .write_secret_file(&certs.server_key, server_key.serialize_pem().as_bytes())?;
        self.system
            .write_file(&certs.client_cert, client_cert.pem().as_bytes())?;
        self.system
            .write_secret_file(&certs.client_key, client_key.serialize_pem().as_bytes())?;

        Ok(certs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_exists_follows_the_ca_cert() {
        let dir = PathBuf::from("/data/machines/test");
        let system = SystemMock::new();
        assert!(!InstanceCertGenerator::new(&system, dir.clone()).exists());

        let system = SystemMock::new().add_file("/data/machines/test/ca-cert.pem", b"cert");
        assert!(InstanceCertGenerator::new(&system, dir).exists());
    }

    #[test]
    fn test_generate_writes_all_five_pem_files() {
        let system = SystemMock::new();
        let generator = InstanceCertGenerator::new(&system, PathBuf::from("/data/machines/test"));

        let certs = generator.generate().unwrap();

        for path in [
            &certs.ca_cert,
            &certs.server_cert,
            &certs.server_key,
            &certs.client_cert,
            &certs.client_key,
        ] {
            let content = system
                .get_written_file(&path.to_string_lossy())
                .unwrap_or_else(|| panic!("expected {path:?} to have been written"));
            assert!(!content.is_empty());
        }
    }
}
