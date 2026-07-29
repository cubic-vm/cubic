use crate::error::{Error, Result};
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
