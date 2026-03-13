use axum::{
    body::Body,
    extract::State,
    http::{ Request, StatusCode },
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;

use crate::state::AppState;

/// Middleware that checks API key if the app is in "external" mode.
pub async fn api_key_layer(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // If not external mode, skip API key check
    if !state.config.is_external() {
        return Ok(next.run(request).await);
    }

    // Skip API key check for auth routes, docs, health, and dev
    let path = request.uri().path();
    if path.starts_with("/api/auth") || path.starts_with("/docs") || path == "/health" || path.starts_with("/dev") {
        return Ok(next.run(request).await);
    }

    // Extract API key from header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if key == state.config.security.api_key => { Ok(next.run(request).await) }
        _ => {
            tracing::warn!("Invalid or missing API key for external request: {}", path);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(
                    json!({
                    "success": false,
                    "error": {
                        "type": "invalid_api_key",
                        "message": "Invalid or missing API key"
                    }
                })
                ),
            ))
        }
    }
}
