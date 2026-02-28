use axum::{ extract::{ Path, Query, State }, routing::{ get, post }, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::Result,
    middleware::{ activity_log::log_activity, auth::FullUser },
    models::mental::GenerateReportRequest,
    services::ReportService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(generate_report))
        .route("/", get(get_reports))
        .route("/{report_id}", get(get_report))
}

#[derive(Debug, Deserialize)]
struct ReportQueryParams {
    limit: Option<i64>,
}

// ── POST /api/reports/generate ──────────────────────────────

async fn generate_report(
    State(state): State<AppState>,
    auth: FullUser,
    Json(req): Json<GenerateReportRequest>
) -> Result<Json<Value>> {
    let report = ReportService::generate_report(
        &state.db,
        &state.crypto,
        &state.gemini,
        &state.email,
        &auth.user.id,
        &auth.user.name,
        &auth.user.email,
        &auth.user.encryption_salt,
        &req,
        "manual"
    ).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "create",
        "reports",
        "mental_report",
        Some(&report.id),
        Some(&format!("type: {}", report.report_type)),
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": report })))
}

// ── GET /api/reports?limit= ────────────────────────────────

async fn get_reports(
    State(state): State<AppState>,
    auth: FullUser,
    Query(params): Query<ReportQueryParams>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(10);

    let reports = ReportService::get_reports(
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
        "data": reports,
        "count": reports.len()
    })
        )
    )
}

// ── GET /api/reports/:report_id ─────────────────────────────

async fn get_report(
    State(state): State<AppState>,
    auth: FullUser,
    Path(report_id): Path<String>
) -> Result<Json<Value>> {
    let report = ReportService::get_report(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &report_id,
        &auth.user.encryption_salt
    ).await?;

    Ok(Json(json!({ "success": true, "data": report })))
}
