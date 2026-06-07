use crate::error::{Error, Result};
use crate::models::InstanceCertPaths;
use rcgen::{
    BasicConstraints, CertificateParams, CertifiedIssuer, DistinguishedName, DnType, IsCa, KeyPair,
    KeyUsagePurpose,
};
use std::fs;
use std::path::PathBuf;

pub struct InstanceCertGenerator {
    dir: PathBuf,
}

impl InstanceCertGenerator {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn generate(&self) -> Result<InstanceCertPaths> {
        let certs = InstanceCertPaths::load(&self.dir);

        let ca_key = KeyPair::generate().map_err(|e| Error::TlsCertGeneration(e.to_string()))?;
        let mut ca_params = CertificateParams::default();

        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(DnType::CommonName, "Cubic CA");
            dn
        };
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca = CertifiedIssuer::self_signed(ca_params, ca_key)
            .map_err(|e| Error::TlsCertGeneration(e.to_string()))?;

        let server_key =
            KeyPair::generate().map_err(|e| Error::TlsCertGeneration(e.to_string()))?;

        let server_cert = CertificateParams::new(vec!["localhost".to_string()])
            .map_err(|e| Error::TlsCertGeneration(e.to_string()))?
            .signed_by(&server_key, &ca)
            .map_err(|e| Error::TlsCertGeneration(e.to_string()))?;

        let client_key =
            KeyPair::generate().map_err(|e| Error::TlsCertGeneration(e.to_string()))?;

        let client_cert = CertificateParams::default()
            .signed_by(&client_key, &ca)
            .map_err(|e| Error::TlsCertGeneration(e.to_string()))?;

        fs::write(&certs.ca_cert, ca.pem()).map_err(Error::from)?;
        fs::write(&certs.server_cert, server_cert.pem()).map_err(Error::from)?;
        fs::write(&certs.server_key, server_key.serialize_pem()).map_err(Error::from)?;
        fs::write(&certs.client_cert, client_cert.pem()).map_err(Error::from)?;
        fs::write(&certs.client_key, client_key.serialize_pem()).map_err(Error::from)?;

        Ok(certs)
    }
}
