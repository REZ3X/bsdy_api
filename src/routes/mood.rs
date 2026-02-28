use axum::{ extract::{ Query, State }, routing::{ get, post }, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::Result,
    middleware::{ activity_log::log_activity, auth::FullUser },
    models::mental::CreateMoodEntryRequest,
    services::MoodService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(upsert_mood))
        .route("/", get(get_mood_entries))
        .route("/today", get(get_today))
}

#[derive(Debug, Deserialize)]
struct MoodQueryParams {
    from: Option<String>,
    to: Option<String>,
    limit: Option<u32>,
}

// ── POST /api/mood ──────────────────────────────────────────

async fn upsert_mood(
    State(state): State<AppState>,
    auth: FullUser,
    Json(req): Json<CreateMoodEntryRequest>
) -> Result<Json<Value>> {
    let entry = MoodService::upsert_mood(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt,
        &req
    ).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "create",
        "mood_tracker",
        "mood_entry",
        Some(&entry.id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": entry })))
}

// ── GET /api/mood?from=&to=&limit= ─────────────────────────

async fn get_mood_entries(
    State(state): State<AppState>,
    auth: FullUser,
    Query(params): Query<MoodQueryParams>
) -> Result<Json<Value>> {
    let from = params.from
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let to = params.to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let entries = MoodService::get_mood_entries(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt,
        from,
        to,
        params.limit
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": entries,
        "count": entries.len()
    })
        )
    )
}

// ── GET /api/mood/today ─────────────────────────────────────

async fn get_today(State(state): State<AppState>, auth: FullUser) -> Result<Json<Value>> {
    let entry = MoodService::get_today(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": entry,
        "logged_today": entry.is_some()
    })
        )
    )
}
