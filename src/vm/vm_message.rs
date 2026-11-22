use serde::{Deserialize, Serialize};
use serde_json::Value;

const JSONRPC_VERSION: &str = "2.0";
const ID: u64 = 1;
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;

#[derive(Debug, Serialize, Deserialize)]
pub struct VmError {
    pub code: i64,
    pub message: String,
}

// Serde tries the variants in order, so a reply never parses as a request
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VmMessage {
    Request {
        jsonrpc: String,
        id: Value,
        method: String,
    },
    Result {
        jsonrpc: String,
        id: Value,
        result: Value,
    },
    Error {
        jsonrpc: String,
        id: Value,
        error: VmError,
    },
}

impl VmMessage {
    pub fn build_request(method: &str) -> Self {
        VmMessage::Request {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: ID.into(),
            method: method.to_string(),
        }
    }

    pub fn build_result() -> Self {
        VmMessage::Result {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: ID.into(),
            result: Value::Null,
        }
    }

    pub fn build_error(code: i64, message: &str) -> Self {
        VmMessage::Error {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: ID.into(),
            error: VmError {
                code,
                message: message.to_string(),
            },
        }
    }
}
