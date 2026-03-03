use chrono::NaiveDateTime;
use serde::Serialize;

// ── Auth Log ────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthLogRow {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct AuthLogResponse {
    pub id: String,
    pub action: String,
    pub ip_address: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

// ── Activity Log ────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ActivityLogRow {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub feature: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct ActivityLogResponse {
    pub id: String,
    pub action: String,
    pub feature: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

// ── Admin Action Log ────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminActionLogRow {
    pub id: String,
    pub admin_id: String,
    pub action: String,
    pub feature: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct AdminActionLogResponse {
    pub id: String,
    pub admin_id: String,
    pub action: String,
    pub feature: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
