use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

// Global registry
static GLOBAL_REGISTRY: OnceLock<Mutex<RpcRegistry>> = OnceLock::new();

pub fn get_registry() -> &'static Mutex<RpcRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| Mutex::new(RpcRegistry::new()))
}

// Basic JSON-RPC structures (same as before)
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    result: Value,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    jsonrpc: String,
    error: JsonRpcErrorDetail,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcErrorDetail {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// RPC Handler trait (same as before)
#[async_trait]
pub trait RpcHandler: Send + Sync {
    async fn handle(
        &self,
        params: Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}

// Modified Registry with method to get all registered handlers
#[derive(Default, Clone)]
pub struct RpcRegistry {
    handlers: HashMap<String, Arc<dyn RpcHandler>>,
}

impl RpcRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<H>(&mut self, method: &str, handler: H)
    where
        H: RpcHandler + 'static,
    {
        self.handlers.insert(method.to_string(), Arc::new(handler));
    }

    // New method to get all handlers
    pub fn into_handler_map(self) -> HashMap<String, Arc<dyn RpcHandler>> {
        self.handlers
    }

    pub async fn handle_request(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        let handler = self
            .handlers
            .get(&request.method)
            .ok_or_else(|| JsonRpcError {
                jsonrpc: "2.0".to_string(),
                error: JsonRpcErrorDetail {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                },
                id: request.id.clone(),
            })?;

        match handler.handle(request.params).await {
            Ok(result) => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result,
                id: request.id,
            }),
            Err(e) => Err(JsonRpcError {
                jsonrpc: "2.0".to_string(),
                error: JsonRpcErrorDetail {
                    code: -32000,
                    message: e.to_string(),
                    data: None,
                },
                id: request.id,
            }),
        }
    }
}

impl JsonRpcError {
    pub fn method_not_found(id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            error: JsonRpcErrorDetail {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            },
            id,
        }
    }

    pub fn invalid_params(id: Option<Value>, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            error: JsonRpcErrorDetail {
                code: -32602,
                message,
                data: None,
            },
            id,
        }
    }

    pub fn internal_error(id: Option<Value>, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            error: JsonRpcErrorDetail {
                code: -32000,
                message,
                data: None,
            },
            id,
        }
    }
}
