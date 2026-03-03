use axum::{ extract::{ Query, State }, routing::get, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::{ AppError, Result },
    middleware::auth::{ AuthUser, AdminUser },
    models::log::{
        ActivityLogResponse,
        AdminActionLogResponse,
        AuthLogResponse,
        PaginatedResponse,
    },
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth", get(get_auth_logs))
        .route("/activity", get(get_activity_logs))
        .route("/admin", get(get_admin_action_logs))
}

#[derive(Debug, Deserialize)]
struct LogQueryParams {
    page: Option<u32>,
    per_page: Option<u32>,
    feature: Option<String>,
}

// ── GET /api/logs/auth?page=&per_page= ─────────────────────

async fn get_auth_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<LogQueryParams>
) -> Result<Json<Value>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let total: (i64,) = sqlx
        ::query_as("SELECT COUNT(*) FROM user_auth_logs WHERE user_id = ?")
        .bind(&auth.user.id)
        .fetch_one(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let rows: Vec<crate::models::log::AuthLogRow> = sqlx
        ::query_as(
            r#"SELECT * FROM user_auth_logs
           WHERE user_id = ?
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#
        )
        .bind(&auth.user.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let data: Vec<AuthLogResponse> = rows
        .iter()
        .map(|r| AuthLogResponse {
            id: r.id.clone(),
            action: r.action.clone(),
            ip_address: r.ip_address.clone(),
            success: r.success,
            failure_reason: r.failure_reason.clone(),
            created_at: r.created_at.to_string(),
        })
        .collect();

    let resp = PaginatedResponse {
        data,
        total: total.0,
        page,
        per_page,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}

// ── GET /api/logs/activity?page=&per_page=&feature= ────────

async fn get_activity_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<LogQueryParams>
) -> Result<Json<Value>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Build WHERE clause
    let (count_query, data_query, has_feature);
    if let Some(ref feature) = params.feature {
        has_feature = true;
        count_query = format!(
            "SELECT COUNT(*) FROM user_activity_logs WHERE user_id = ? AND feature = '{}'",
            feature.replace('\'', "''")
        );
        data_query = format!(
            r#"SELECT * FROM user_activity_logs
               WHERE user_id = ? AND feature = '{}'
               ORDER BY created_at DESC
               LIMIT ? OFFSET ?"#,
            feature.replace('\'', "''")
        );
    } else {
        has_feature = false;
        count_query = "SELECT COUNT(*) FROM user_activity_logs WHERE user_id = ?".to_string();
        data_query =
            r#"SELECT * FROM user_activity_logs
           WHERE user_id = ?
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#.to_string();
    }
    // Suppress unused variable warning
    let _ = has_feature;

    let total: (i64,) = sqlx
        ::query_as(&count_query)
        .bind(&auth.user.id)
        .fetch_one(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let rows: Vec<crate::models::log::ActivityLogRow> = sqlx
        ::query_as(&data_query)
        .bind(&auth.user.id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let data: Vec<ActivityLogResponse> = rows
        .iter()
        .map(|r| ActivityLogResponse {
            id: r.id.clone(),
            action: r.action.clone(),
            feature: r.feature.clone(),
            entity_type: r.entity_type.clone(),
            entity_id: r.entity_id.clone(),
            details: r.details.clone(),
            created_at: r.created_at.to_string(),
        })
        .collect();

    let resp = PaginatedResponse {
        data,
        total: total.0,
        page,
        per_page,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}

// ── GET /api/logs/admin?page=&per_page=&feature= (admin only) ──

async fn get_admin_action_logs(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(params): Query<LogQueryParams>
) -> Result<Json<Value>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // Build WHERE clause
    let (count_query, data_query);
    if let Some(ref feature) = params.feature {
        count_query = format!(
            "SELECT COUNT(*) FROM admin_action_logs WHERE feature = '{}'",
            feature.replace('\'', "''")
        );
        data_query = format!(
            r#"SELECT * FROM admin_action_logs
               WHERE feature = '{}'
               ORDER BY created_at DESC
               LIMIT ? OFFSET ?"#,
            feature.replace('\'', "''")
        );
    } else {
        count_query = "SELECT COUNT(*) FROM admin_action_logs".to_string();
        data_query =
            r#"SELECT * FROM admin_action_logs
           ORDER BY created_at DESC
           LIMIT ? OFFSET ?"#.to_string();
    }

    let total: (i64,) = sqlx
        ::query_as(&count_query)
        .fetch_one(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let rows: Vec<crate::models::log::AdminActionLogRow> = sqlx
        ::query_as(&data_query)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let data: Vec<AdminActionLogResponse> = rows
        .iter()
        .map(|r| AdminActionLogResponse {
            id: r.id.clone(),
            admin_id: r.admin_id.clone(),
            action: r.action.clone(),
            feature: r.feature.clone(),
            entity_type: r.entity_type.clone(),
            entity_id: r.entity_id.clone(),
            details: r.details.clone(),
            created_at: r.created_at.to_string(),
        })
        .collect();

    let resp = PaginatedResponse {
        data,
        total: total.0,
        page,
        per_page,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}
