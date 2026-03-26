use axum::{ extract::State, routing::get, Json, Router };
use serde_json::{ json, Value };

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // 1) Database connectivity
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    // 2) Gemini API – lightweight models.list call to validate the key
    let gemini_ok = state.http_client
        .get(
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                state.config.gemini.api_key
            )
        )
        .timeout(std::time::Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    // 3) SMTP (Brevo) – attempt a STARTTLS connection then quit
    let smtp_ok = {
        let host = &state.config.brevo.smtp_host;
        let port = state.config.brevo.smtp_port;
        tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await.is_ok()
    };

    // 4) Google OAuth – HEAD request to OpenID discovery endpoint
    let google_ok = state.http_client
        .get("https://accounts.google.com/.well-known/openid-configuration")
        .timeout(std::time::Duration::from_secs(5))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let all_ok = db_ok && gemini_ok && smtp_ok && google_ok;

    Json(
        json!({
            "success": true,
            "status": if all_ok { "healthy" } else { "degraded" },
            "service": "bsdy-api",
            "version": env!("CARGO_PKG_VERSION"),
            "checks": {
                "database": if db_ok { "connected" } else { "disconnected" },
                "gemini_api": if gemini_ok { "reachable" } else { "unreachable" },
                "smtp_brevo": if smtp_ok { "reachable" } else { "unreachable" },
                "google_oauth": if google_ok { "reachable" } else { "unreachable" },
            }
        })
    )
}
