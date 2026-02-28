use chrono::NaiveDateTime;
use serde::{ Deserialize, Serialize };

// ── Note (Coping Toolkit) ───────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NoteRow {
    pub id: String,
    pub user_id: String,
    pub title_enc: String,
    pub content_enc: String,
    pub label: Option<String>,
    pub is_pinned: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct NoteResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub label: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub label: Option<String>,
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub label: Option<String>,
    pub is_pinned: Option<bool>,
}
