use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

use crate::{AppError, AppState};

#[axum::debug_handler] // super helpful for debugging
pub async fn echo_json(
    State(_state): State<AppState>,
    payload: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("echo_json");
    Ok(payload)
}

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct EchoJSONExtractorValue {
    #[validate(length(min = 3, max = 10))]
    name: String,
}

pub async fn echo_json_extractor(
    State(_state): State<AppState>,
    Json(payload): Json<EchoJSONExtractorValue>,
) -> Result<Json<EchoJSONExtractorValue>, AppError> {
    match payload.validate() {
        Ok(_) => Ok(payload.into()),
        Err(e) => Err(AppError::ValidationError(e)),
    }
}
