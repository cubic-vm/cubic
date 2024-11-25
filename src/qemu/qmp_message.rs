use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct QmpTimestamp {
    seconds: i64,
    microseconds: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QmpError {
    class: String,
    desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QmpMessage {
    Greeting {
        #[serde(alias = "QMP")]
        qmp: Value,
    },

    Command {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        execute: String,
        #[serde(skip_serializing_if = "Value::is_null")]
        arguments: Value,
    },

    Event {
        event: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
        timestamp: QmpTimestamp,
    },

    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(alias = "return")]
        ret: Value,
    },

    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        error: QmpError,
    },
}
