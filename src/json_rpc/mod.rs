use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC Error Codes
pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

// Basic JSON-RPC structures (same as before)
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub enum JsonRpcResponse<T: Serialize> {
    Success {
        jsonrpc: String,
        result: Value,
        id: Option<Value>,
    },
    Error {
        jsonrpc: String,
        id: Option<Value>,
        data: Option<T>,
        code: i64,
    },
}

#[derive(Debug, Serialize)]
pub struct InternalError {
    pub message: String,
}

impl From<anyhow::Error> for InternalError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}
