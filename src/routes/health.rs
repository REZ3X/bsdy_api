use axum::{ extract::State, routing::get, Json, Router };
use serde_json::{ json, Value };

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // Quick DB connectivity check
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    Json(
        json!({
        "success": true,
        "status": if db_ok { "healthy" } else { "degraded" },
        "service": "bsdy-api",
        "version": env!("CARGO_PKG_VERSION"),
        "database": if db_ok { "connected" } else { "disconnected" },
    })
    )
}
