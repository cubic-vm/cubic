use std::path::{Path, PathBuf};

pub struct InstanceCertPaths {
    pub ca_cert: PathBuf,
    pub server_cert: PathBuf,
    pub server_key: PathBuf,
    pub client_cert: PathBuf,
    pub client_key: PathBuf,
}

impl InstanceCertPaths {
    pub fn load(dir: &Path) -> Self {
        Self {
            ca_cert: dir.join("ca-cert.pem"),
            server_cert: dir.join("server-cert.pem"),
            server_key: dir.join("server-key.pem"),
            client_cert: dir.join("client-cert.pem"),
            client_key: dir.join("client-key.pem"),
        }
    }

    pub fn exists(&self) -> bool {
        self.ca_cert.exists()
    }
}
