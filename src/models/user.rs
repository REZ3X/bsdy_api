use chrono::NaiveDateTime;
use serde::{ Deserialize, Serialize };

// ── Database Row ────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub google_id: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub birth: Option<chrono::NaiveDate>,
    pub email_verification_status: String,
    pub email_verification_token: Option<String>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub onboarding_completed: bool,
    pub encryption_salt: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ── API DTOs ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub birth: Option<String>,
    pub email_verified: bool,
    pub onboarding_completed: bool,
    pub created_at: String,
}

impl From<&UserRow> for UserResponse {
    fn from(u: &UserRow) -> Self {
        Self {
            id: u.id.clone(),
            username: u.username.clone(),
            name: u.name.clone(),
            email: u.email.clone(),
            avatar_url: u.avatar_url.clone(),
            birth: u.birth.map(|b| b.to_string()),
            email_verified: u.email_verification_status == "verified",
            onboarding_completed: u.onboarding_completed,
            created_at: u.created_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
    pub is_new_user: bool,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCallbackRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub birth: Option<String>, // YYYY-MM-DD
}
