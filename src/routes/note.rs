use axum::{ extract::{ Path, Query, State }, routing::{ delete, get, post, put }, Json, Router };
use serde::Deserialize;
use serde_json::{ json, Value };

use crate::{
    error::Result,
    middleware::{ activity_log::log_activity, auth::FullUser },
    models::note::{ CreateNoteRequest, UpdateNoteRequest },
    services::NoteService,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_note))
        .route("/", get(get_notes))
        .route("/labels", get(get_labels))
        .route("/{note_id}", get(get_note))
        .route("/{note_id}", put(update_note))
        .route("/{note_id}", delete(delete_note))
}

#[derive(Debug, Deserialize)]
struct NoteQueryParams {
    label: Option<String>,
    limit: Option<i64>,
}

// ── POST /api/notes ─────────────────────────────────────────

async fn create_note(
    State(state): State<AppState>,
    auth: FullUser,
    Json(req): Json<CreateNoteRequest>
) -> Result<Json<Value>> {
    let note = NoteService::create_note(
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
        "coping_toolkit",
        "note",
        Some(&note.id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": note })))
}

// ── GET /api/notes?label=&limit= ───────────────────────────

async fn get_notes(
    State(state): State<AppState>,
    auth: FullUser,
    Query(params): Query<NoteQueryParams>
) -> Result<Json<Value>> {
    let limit = params.limit.unwrap_or(50);

    let notes = NoteService::get_notes(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &auth.user.encryption_salt,
        params.label.as_deref(),
        limit
    ).await?;

    Ok(
        Json(
            json!({
        "success": true,
        "data": notes,
        "count": notes.len()
    })
        )
    )
}

// ── GET /api/notes/labels ───────────────────────────────────

async fn get_labels(State(state): State<AppState>, auth: FullUser) -> Result<Json<Value>> {
    let labels = NoteService::get_labels(&state.db, &auth.user.id).await?;

    Ok(Json(json!({
        "success": true,
        "data": labels
    })))
}

// ── GET /api/notes/:note_id ─────────────────────────────────

async fn get_note(
    State(state): State<AppState>,
    auth: FullUser,
    Path(note_id): Path<String>
) -> Result<Json<Value>> {
    let note = NoteService::get_note(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &note_id,
        &auth.user.encryption_salt
    ).await?;

    Ok(Json(json!({ "success": true, "data": note })))
}

// ── PUT /api/notes/:note_id ────────────────────────────────

async fn update_note(
    State(state): State<AppState>,
    auth: FullUser,
    Path(note_id): Path<String>,
    Json(req): Json<UpdateNoteRequest>
) -> Result<Json<Value>> {
    let note = NoteService::update_note(
        &state.db,
        &state.crypto,
        &auth.user.id,
        &note_id,
        &auth.user.encryption_salt,
        &req
    ).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "update",
        "coping_toolkit",
        "note",
        Some(&note_id),
        None,
        None
    ).await;

    Ok(Json(json!({ "success": true, "data": note })))
}

// ── DELETE /api/notes/:note_id ──────────────────────────────

async fn delete_note(
    State(state): State<AppState>,
    auth: FullUser,
    Path(note_id): Path<String>
) -> Result<Json<Value>> {
    NoteService::delete_note(&state.db, &auth.user.id, &note_id).await?;

    log_activity(
        &state.db,
        &auth.user.id,
        "delete",
        "coping_toolkit",
        "note",
        Some(&note_id),
        None,
        None
    ).await;

    Ok(Json(json!({
        "success": true,
        "message": "Note deleted"
    })))
}
