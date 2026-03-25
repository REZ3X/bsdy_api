use axum::{response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/dev", get(dev_page))
}

async fn dev_page() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "code": 777,
        "data": {
            "message": ["Slaviors - REZ3X"],
            "credits": {
                "built_by": "REZ3X",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    }))
}
