pub mod auth;
pub mod onboarding;
pub mod mood;
pub mod analytics;
pub mod report;
pub mod note;
pub mod chat;
pub mod content;
pub mod log;
pub mod health;
pub mod docs;
pub mod dev;

use axum::Router;
use tower_http::services::ServeDir;

use crate::state::AppState;

/// Router builder
pub fn build_router() -> Router<AppState> {
    Router::new()
        .nest("/api/auth", auth::routes())
        .nest("/api/onboarding", onboarding::routes())
        .nest("/api/mood", mood::routes())
        .nest("/api/analytics", analytics::routes())
        .nest("/api/reports", report::routes())
        .nest("/api/notes", note::routes())
        .nest("/api/chats", chat::routes())
        .nest("/api/content", content::routes())
        .nest("/api/logs", log::routes())
        .merge(health::routes())
        .merge(docs::routes())
        .merge(dev::routes())
        .nest_service("/uploads", ServeDir::new("uploads"))
}
