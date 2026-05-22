use crate::models::Instance;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: Instance,
}
