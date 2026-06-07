use crate::error::{Error, Result};
use crate::models::InstanceCertPaths;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::net::TcpStream;
use std::sync::Arc;

pub struct TlsClient {
    config: Arc<ClientConfig>,
}

impl TlsClient {
    pub fn new(certs: &InstanceCertPaths) -> Result<Self> {
        let ca_der = CertificateDer::from_pem_file(&certs.ca_cert)
            .map_err(|e| Error::TlsConnection(e.to_string()))?;

        let mut root_store = RootCertStore::empty();
        root_store
            .add(ca_der)
            .map_err(|e| Error::TlsConnection(e.to_string()))?;

        let client_certs = CertificateDer::pem_file_iter(&certs.client_cert)
            .map_err(|e| Error::TlsConnection(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::TlsConnection(e.to_string()))?;

        let client_key = PrivateKeyDer::from_pem_file(&certs.client_key)
            .map_err(|e| Error::TlsConnection(e.to_string()))?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(client_certs, client_key)
            .map_err(|e| Error::TlsConnection(e.to_string()))?;

        Ok(Self {
            config: Arc::new(config),
        })
    }

    pub fn connect(&self, port: u16) -> Result<StreamOwned<ClientConnection, TcpStream>> {
        let server_name =
            ServerName::try_from("localhost").map_err(|e| Error::TlsConnection(e.to_string()))?;
        let conn = ClientConnection::new(self.config.clone(), server_name)
            .map_err(|e| Error::TlsConnection(e.to_string()))?;
        let tcp = TcpStream::connect(format!("127.0.0.1:{port}")).map_err(Error::from)?;
        Ok(StreamOwned::new(conn, tcp))
    }
}
