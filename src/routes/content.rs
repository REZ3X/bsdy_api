use axum::{
    extract::{ DefaultBodyLimit, Multipart, Path, Query, State },
    routing::{ delete, get, post, put },
    Json,
    Router,
};
use serde::Deserialize;
use serde_json::{ json, Value };
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::{ AppError, Result },
    middleware::{ activity_log::log_activity, auth::{ AdminUser, AuthUser } },
    models::content::{ CreateContentRequest, UpdateContentRequest },
    services::ContentService,
    state::AppState,
};

/// Upload directory relative to the executable working directory.
const UPLOAD_DIR: &str = "uploads/content";

/// Max upload size: 10 MB.
const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;

/// Allowed image MIME types.
const ALLOWED_MIME: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Build admin content management routes (CRUD) and public read routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Public read routes (any authenticated user can read published content)
        .route("/", get(list_contents))
        .route("/{content_id}", get(get_content))
        .route("/slug/{slug}", get(get_content_by_slug))
        // Admin-only management routes
        .route("/", post(create_content))
        .route("/{content_id}", put(update_content))
        .route("/{content_id}", delete(delete_content))
        .route("/{content_id}/cover", post(upload_cover_image))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Resolve the image base URL from config (e.g. "http://localhost:8000").
fn image_base_url(state: &AppState) -> String {
    if state.config.app.env == "production" {
        state.config.app.frontend_url.clone()
    } else {
        format!("http://localhost:{}", state.config.app.port)
    }
}

/// Ensure the upload directory exists.
async fn ensure_upload_dir() -> Result<PathBuf> {
    let dir = PathBuf::from(UPLOAD_DIR);
    fs::create_dir_all(&dir).await.map_err(|e| {
        tracing::error!("Failed to create upload directory: {}", e);
        AppError::InternalError(anyhow::anyhow!("Failed to create upload directory: {}", e))
    })?;
    Ok(dir)
}

// ── GET /api/content?limit=&offset= ────────────────────────
// Public: lists published content. Admin: lists all.

async fn list_contents(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    Query(params): Query<ListQuery>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    // Check if the caller is an admin
    let is_admin = auth
        .as_ref()
        .map(|a| a.user.role == "admin")
        .unwrap_or(false);

    let base = image_base_url(&state);
    let (items, total) = ContentService::list_contents(
        &state.db,
        is_admin,
        limit,
        offset,
        &base
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": items,
        "total": total,
        "limit": limit,
        "offset": offset
    })
        )
    )
}

// ── GET /api/content/:content_id ────────────────────────────

async fn get_content(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    Path(content_id): Path<String>
) -> Result<Json<Value>> {
    let is_admin = auth
        .as_ref()
        .map(|a| a.user.role == "admin")
        .unwrap_or(false);

    let base = image_base_url(&state);
    let content = ContentService::get_content(&state.db, &content_id, is_admin, &base).await?;

    Ok(Json(json!({ "success": true, "data": content })))
}

// ── GET /api/content/slug/:slug ─────────────────────────────

async fn get_content_by_slug(
    State(state): State<AppState>,
    auth: Option<AuthUser>,
    Path(slug): Path<String>
) -> Result<Json<Value>> {
    let is_admin = auth
        .as_ref()
        .map(|a| a.user.role == "admin")
        .unwrap_or(false);

    let base = image_base_url(&state);
    let content = ContentService::get_content_by_slug(&state.db, &slug, is_admin, &base).await?;

    Ok(Json(json!({ "success": true, "data": content })))
}

// ── POST /api/content (admin only) ─────────────────────────

async fn create_content(
    State(state): State<AppState>,
    admin: AdminUser,
    Json(req): Json<CreateContentRequest>
) -> Result<Json<Value>> {
    let base = image_base_url(&state);
    let content = ContentService::create_content(&state.db, &admin.user.id, &req, &base).await?;

    log_activity(
        &state.db,
        &admin.user.id,
        "create",
        "content",
        "content",
        Some(&content.id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": content })))
}

// ── PUT /api/content/:content_id (admin only) ──────────────

async fn update_content(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(content_id): Path<String>,
    Json(req): Json<UpdateContentRequest>
) -> Result<Json<Value>> {
    let base = image_base_url(&state);
    let content = ContentService::update_content(&state.db, &content_id, &req, &base).await?;

    log_activity(
        &state.db,
        &admin.user.id,
        "update",
        "content",
        "content",
        Some(&content_id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": content })))
}

// ── DELETE /api/content/:content_id (admin only) ────────────

async fn delete_content(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(content_id): Path<String>
) -> Result<Json<Value>> {
    // Try to delete cover image from disk
    if
        let Ok(row) = sqlx
            ::query_as::<_, (Option<String>,)>("SELECT cover_image FROM contents WHERE id = ?")
            .bind(&content_id)
            .fetch_one(&state.db).await
    {
        if let Some(filename) = row.0 {
            let path = PathBuf::from(UPLOAD_DIR).join(&filename);
            let _ = fs::remove_file(&path).await;
        }
    }

    ContentService::delete_content(&state.db, &content_id).await?;

    log_activity(
        &state.db,
        &admin.user.id,
        "delete",
        "content",
        "content",
        Some(&content_id),
        None,
        None
    ).await;

    Ok(Json(json!({
        "success": true,
        "message": "Content deleted"
    })))
}

// ── POST /api/content/:content_id/cover (admin only) ───────
// Multipart file upload for cover image.

async fn upload_cover_image(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(content_id): Path<String>,
    mut multipart: Multipart
) -> Result<Json<Value>> {
    // Verify content exists
    let existing: Option<(Option<String>,)> = sqlx
        ::query_as("SELECT cover_image FROM contents WHERE id = ?")
        .bind(&content_id)
        .fetch_optional(&state.db).await
        .map_err(AppError::DatabaseError)?;

    let old_image = existing.ok_or_else(|| AppError::NotFound("Content not found".into()))?.0;

    let upload_dir = ensure_upload_dir().await?;

    // Process the multipart field
    let field = multipart
        .next_field().await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart data: {}", e)))?
        .ok_or_else(|| AppError::BadRequest("No file field provided".into()))?;

    // Validate content type
    let content_type = field
        .content_type()
        .ok_or_else(|| AppError::BadRequest("Missing content type".into()))?
        .to_string();

    if !ALLOWED_MIME.contains(&content_type.as_str()) {
        return Err(
            AppError::BadRequest(
                format!("Invalid image type '{}'. Allowed: JPEG, PNG, WebP, GIF", content_type)
            )
        );
    }

    // Determine extension from MIME type
    let ext = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "bin",
    };

    // Read file bytes
    let data = field
        .bytes().await
        .map_err(|e| AppError::BadRequest(format!("Failed to read upload: {}", e)))?;

    if data.is_empty() {
        return Err(AppError::BadRequest("Uploaded file is empty".into()));
    }

    // Generate unique filename
    let filename = format!("{}_{}.{}", content_id, Uuid::new_v4(), ext);
    let file_path = upload_dir.join(&filename);

    // Write file to disk
    fs::write(&file_path, &data).await.map_err(|e| {
        tracing::error!("Failed to write image file: {}", e);
        AppError::InternalError(anyhow::anyhow!("Failed to save image: {}", e))
    })?;

    // Delete old cover image if it exists
    if let Some(old) = old_image {
        let old_path = upload_dir.join(&old);
        let _ = fs::remove_file(&old_path).await;
    }

    // Update database with new filename
    let base = image_base_url(&state);
    let content = ContentService::set_cover_image(&state.db, &content_id, &filename, &base).await?;

    log_activity(
        &state.db,
        &admin.user.id,
        "update",
        "content",
        "content_cover",
        Some(&content_id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": content })))
}
