use axum::{extract::State, response::Response, Json};

use crate::{AppError, AppState};

pub async fn echo_json(
    State(state): State<AppState>,
    payload: Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(payload)
}
