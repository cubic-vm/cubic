use crate::instance::InstanceStore;
use crate::models::{Instance, OsRelease};
use crate::ssh::SftpPath;
use crate::view::Console;
use russh::{Channel, client};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

const OS_RELEASE_FILE: &str = "/etc/os-release";
/// The file holds a handful of short lines, so anything larger is broken.
const OS_RELEASE_MAX_BYTES: u64 = 8 * 1024;
const READ_TIMEOUT_SECS: u64 = 5;

/// Reads the OS of a guest over SFTP and keeps the instance config current.
pub struct GuestOsReader<'a> {
    console: &'a Arc<Console>,
    store: &'a dyn InstanceStore,
}

impl<'a> GuestOsReader<'a> {
    pub fn new(console: &'a Arc<Console>, store: &'a dyn InstanceStore) -> Self {
        Self { console, store }
    }

    /// Stores the OS of the guest when it differs from the value in the
    /// config. The read is best effort, so a guest that keeps the file to
    /// itself keeps the stored value and connects as usual.
    pub async fn refresh(&self, channel: Channel<client::Msg>, instance: &mut Instance) {
        let read = Self::read(channel, &instance.name);

        let Ok(Some(os)) = tokio::time::timeout(Duration::from_secs(READ_TIMEOUT_SECS), read).await
        else {
            self.console.debug(&format!(
                "Reading the guest OS of '{}' failed, keeping the stored value",
                instance.name
            ));
            return;
        };

        if instance.os.as_deref() == Some(os.as_str()) {
            self.console
                .debug(&format!("Guest OS of '{}' is {os}", instance.name));
            return;
        }

        self.console
            .debug(&format!("Guest OS of '{}' changed to {os}", instance.name));
        instance.os = Some(os);
        self.store.store(instance).ok();
    }

    /// Returns nothing when the file is missing, unreadable or carries no ID.
    async fn read(channel: Channel<client::Msg>, machine: &str) -> Option<String> {
        let path = SftpPath {
            sftp: Some(SftpPath::start_session(machine, channel).await.ok()?),
            path: PathBuf::from(OS_RELEASE_FILE),
        };

        let mut content = String::new();
        path.open_file()
            .await
            .ok()?
            .take(OS_RELEASE_MAX_BYTES)
            .read_to_string(&mut content)
            .await
            .ok()?;

        OsRelease::parse_name(&content)
    }
}
