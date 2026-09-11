use crate::models::{Arch, DataSize, PortForward, Snapshot, UserName};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub arch: Arch,
    #[serde(default)]
    pub user: UserName,
    pub cpus: u16,
    pub mem: DataSize,
    #[serde(skip)]
    pub disk_used: Option<DataSize>,
    /// Read back from the disk image, never written to the config
    #[serde(skip)]
    pub snapshots: Vec<Snapshot>,
    pub disk_capacity: DataSize,
    pub ssh_port: u16,
    #[serde(default)]
    pub monitor_port: Option<u16>,
    #[serde(default)]
    pub console_port: Option<u16>,
    #[serde(default)]
    pub hostfwd: Vec<PortForward>,
    #[serde(default)]
    pub execute: Option<String>,
    #[serde(default)]
    pub isolate: bool,
    /// Guest SSH host key, pinned on the first connect
    #[serde(default)]
    pub ssh_host_key: Option<String>,
    /// Set by `cubic run --rm`, the instance is deleted once it stops
    #[serde(default)]
    pub auto_remove: bool,
}

impl Instance {
    pub fn has_snapshot(&self, name: &str) -> bool {
        self.snapshots.iter().any(|snapshot| snapshot.name == name)
    }
}
