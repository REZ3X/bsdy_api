use axum::{ extract::{ Query, State }, routing::{ get, post, put }, Json, Router };
use serde_json::{ json, Value };

use crate::{
    error::{ AppError, Result },
    middleware::{
        activity_log::{ log_activity, log_auth_event },
        auth::{ AuthUser, VerifiedUser },
    },
    models::{
        AuthResponse,
        GoogleCallbackRequest,
        UpdateProfileRequest,
        UserResponse,
        VerifyEmailQuery,
    },
    services::AuthService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/google/url", get(google_auth_url))
        .route("/google/callback", post(google_callback))
        .route("/verify-email", get(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/me", get(get_me))
        .route("/me", put(update_profile))
}

// ── GET /api/auth/google/url ────────────────────────────────

async fn google_auth_url(State(state): State<AppState>) -> Result<Json<Value>> {
    let url = AuthService::google_auth_url(&state.config)?;
    Ok(Json(json!({ "success": true, "url": url })))
}

// ── POST /api/auth/google/callback ──────────────────────────

async fn google_callback(
    State(state): State<AppState>,
    Json(req): Json<GoogleCallbackRequest>
) -> Result<Json<Value>> {
    let google_user = AuthService::exchange_code(
        &req.code,
        &state.config,
        &state.http_client
    ).await?;

    let (user, is_new) = AuthService::find_or_create_user(
        &state.db,
        &google_user,
        &state.crypto
    ).await?;

    let token = AuthService::generate_jwt(&user, &state.config)?;

    // Log auth event
    log_auth_event(&state.db, &user.id, "login", None, None, true, None).await;

    // Send verification email for new users
    if is_new {
        if let Some(ref vtoken) = user.email_verification_token {
            let verify_url = format!(
                "{}/api/auth/verify-email?token={}",
                state.config.app.frontend_url,
                vtoken
            );
            let _ = state.email.send_verification_email(&user.email, &user.name, &verify_url).await;
            log_auth_event(&state.db, &user.id, "verification_sent", None, None, true, None).await;
        }
    }

    let resp = AuthResponse {
        token,
        user: UserResponse::from(&user),
        is_new_user: is_new,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}

// ── GET /api/auth/verify-email?token=... ────────────────────

async fn verify_email(
    State(state): State<AppState>,
    Query(q): Query<VerifyEmailQuery>
) -> Result<Json<Value>> {
    let user = AuthService::verify_email(&state.db, &q.token).await?;

    log_auth_event(&state.db, &user.id, "email_verify", None, None, true, None).await;

    Ok(
        Json(
            json!({
        "success": true,
        "message": "Email verified successfully",
        "user": UserResponse::from(&user)
    })
        )
    )
}

// ── POST /api/auth/resend-verification ──────────────────────

async fn resend_verification(State(state): State<AppState>, auth: AuthUser) -> Result<Json<Value>> {
    if auth.user.email_verification_status == "verified" {
        return Err(AppError::Conflict("Email is already verified".into()));
    }

    let new_token = AuthService::generate_verification_token();

    sqlx
        ::query("UPDATE users SET email_verification_token = ?, updated_at = NOW() WHERE id = ?")
        .bind(&new_token)
        .bind(&auth.user.id)
        .execute(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let verify_url = format!(
        "{}/api/auth/verify-email?token={}",
        state.config.app.frontend_url,
        new_token
    );
    state.email
        .send_verification_email(&auth.user.email, &auth.user.name, &verify_url).await
        .map_err(|e| AppError::InternalError(e.into()))?;

    log_auth_event(&state.db, &auth.user.id, "verification_sent", None, None, true, None).await;

    Ok(Json(json!({
        "success": true,
        "message": "Verification email sent"
    })))
}

// ── GET /api/auth/me ────────────────────────────────────────

async fn get_me(auth: AuthUser) -> Result<Json<Value>> {
    Ok(Json(json!({
        "success": true,
        "data": UserResponse::from(&auth.user)
    })))
}

// ── PUT /api/auth/me ────────────────────────────────────────

async fn update_profile(
    State(state): State<AppState>,
    auth: VerifiedUser,
    Json(req): Json<UpdateProfileRequest>
) -> Result<Json<Value>> {
    // Build update query dynamically
    let mut updates = Vec::new();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(AppError::ValidationError("Name cannot be empty".into()));
        }
        updates.push("name = ?");
        binds.push(name.trim().to_string());
    }

    if let Some(ref birth) = req.birth {
        // Validate date format
        chrono::NaiveDate
            ::parse_from_str(birth, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError("Invalid date format. Use YYYY-MM-DD".into()))?;
        updates.push("birth = ?");
        binds.push(birth.clone());
    }

    if updates.is_empty() {
        return Err(AppError::ValidationError("At least one field must be provided".into()));
    }

    updates.push("updated_at = NOW()");
    let query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));

    let mut q = sqlx::query(&query);
    for val in &binds {
        q = q.bind(val);
    }
    q = q.bind(&auth.user.id);
    q.execute(&state.db).await.map_err(AppError::DatabaseError)?;

    // Fetch updated user
    let user: crate::models::UserRow = sqlx
        ::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&auth.user.id)
        .fetch_one(&state.db).await
        .map_err(AppError::DatabaseError)?;

    log_activity(
        &state.db,
        &auth.user.id,
        "update",
        "profile",
        "user",
        Some(&auth.user.id),
        None,
        None
    ).await;

    Ok(Json(json!({
        "success": true,
        "data": UserResponse::from(&user)
    })))
}
