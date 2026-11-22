use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VmState {
    Starting,
    Running,
    Stopping,
    Stopped,
    #[serde(other)]
    Unknown,
}

impl VmState {
    pub fn get_name(&self) -> &'static str {
        match self {
            VmState::Starting => "starting",
            VmState::Running => "running",
            VmState::Stopping => "stopping",
            VmState::Stopped => "stopped",
            VmState::Unknown => "unknown",
        }
    }
}

pub trait VmControl {
    fn stop(&mut self) -> Result<()>;
    fn quit(&mut self) -> Result<()>;
    fn get_state(&mut self) -> Result<VmState>;
}
