use axum::{ extract::State, routing::{ get, post, put }, Json, Router };
use serde_json::{ json, Value };

use crate::{
    error::Result,
    middleware::{ activity_log::log_activity, auth::VerifiedUser },
    models::mental::{ BaselineAssessmentRequest, UpdateBaselineRequest },
    services::OnboardingService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/baseline", post(save_baseline))
        .route("/baseline", get(get_baseline))
        .route("/baseline", put(update_baseline))
}

// ── POST /api/onboarding/baseline ───────────────────────────

async fn save_baseline(
    State(state): State<AppState>,
    auth: VerifiedUser,
    Json(req): Json<BaselineAssessmentRequest>
) -> Result<Json<Value>> {
    let baseline = OnboardingService::save_baseline(
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
        "onboarding",
        "mental_characteristics",
        Some(&baseline.id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": baseline })))
}

// ── GET /api/onboarding/baseline ────────────────────────────

async fn get_baseline(State(state): State<AppState>, auth: VerifiedUser) -> Result<Json<Value>> {
    let baseline = OnboardingService::get_baseline(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt
    ).await?;

    Ok(Json(json!({ "success": true, "data": baseline })))
}

// ── PUT /api/onboarding/baseline ────────────────────────────

async fn update_baseline(
    State(state): State<AppState>,
    auth: VerifiedUser,
    Json(req): Json<UpdateBaselineRequest>
) -> Result<Json<Value>> {
    let baseline = OnboardingService::update_baseline(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt,
        &req
    ).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "update",
        "onboarding",
        "mental_characteristics",
        Some(&baseline.id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": baseline })))
}
