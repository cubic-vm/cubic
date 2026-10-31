use crate::platform::file_system_mock::FileSystemMock;
use crate::platform::host_mock::HostMock;
use crate::platform::network_mock::NetworkMock;
use crate::platform::process_mock::CommandMock;
use crate::platform::terminal_mock::TerminalMock;
use std::sync::{Arc, Mutex};

// A host built from one mock per resource. Every resource is implemented in
// the file that owns its mock, so this holds the state and nothing else.
#[derive(Default)]
pub struct SystemMock {
    pub host: HostMock,
    pub terminal: Mutex<TerminalMock>,
    // Shared, so a writer handed out by `create_file` keeps writing into the
    // same files the mock reads back.
    pub file_system: Arc<Mutex<FileSystemMock>>,
    pub commands: Mutex<CommandMock>,
    pub network: Mutex<NetworkMock>,
}

impl SystemMock {
    pub fn new() -> Self {
        Self::default()
    }
}
