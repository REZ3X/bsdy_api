pub mod auth;
pub mod onboarding;
pub mod mood;
pub mod analytics;
pub mod report;
pub mod note;
pub mod chat;
pub mod log;
pub mod health;

use axum::Router;

use crate::state::AppState;

/// Build the complete API router with all route groups.
pub fn build_router() -> Router<AppState> {
    Router::new()
        .nest("/api/auth", auth::routes())
        .nest("/api/onboarding", onboarding::routes())
        .nest("/api/mood", mood::routes())
        .nest("/api/analytics", analytics::routes())
        .nest("/api/reports", report::routes())
        .nest("/api/notes", note::routes())
        .nest("/api/chats", chat::routes())
        .nest("/api/logs", log::routes())
        .merge(health::routes())
}
