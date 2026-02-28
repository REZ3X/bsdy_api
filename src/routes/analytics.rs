use axum::{ extract::{ Query, State }, routing::{ get, post }, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::Result,
    middleware::{ activity_log::log_activity, auth::FullUser },
    models::mental::GenerateAnalyticsRequest,
    services::AnalyticsService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/generate", post(generate_analytics)).route("/", get(get_summaries))
}

#[derive(Debug, Deserialize)]
struct SummaryQueryParams {
    limit: Option<i64>,
}

// ── POST /api/analytics/generate ────────────────────────────

async fn generate_analytics(
    State(state): State<AppState>,
    auth: FullUser,
    Json(req): Json<GenerateAnalyticsRequest>
) -> Result<Json<Value>> {
    let period = req.period_type.as_deref().unwrap_or("weekly");

    let summary = AnalyticsService::generate_summary(
        &state.db,
        &state.crypto,
        &state.gemini,
        &auth.user.id,
        &auth.user.name,
        &auth.user.encryption_salt,
        period,
        "manual"
    ).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "create",
        "analytics",
        "analytics_summary",
        Some(&summary.id),
        Some(&format!("period: {}", period)),
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": summary })))
}

// ── GET /api/analytics?limit= ──────────────────────────────

async fn get_summaries(
    State(state): State<AppState>,
    auth: FullUser,
    Query(params): Query<SummaryQueryParams>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(10);

    let summaries = AnalyticsService::get_summaries(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt,
        limit
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": summaries,
        "count": summaries.len()
    })
        )
    )
}
