use axum::{ http::StatusCode, response::{ IntoResponse, Response }, Json };
use serde_json::json;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")] BadRequest(String),

    #[error("Unauthorized: {0}")] Unauthorized(String),

    #[error("Forbidden: {0}")] Forbidden(String),

    #[error("Not found: {0}")] NotFound(String),

    #[error("Conflict: {0}")] Conflict(String),

    #[error("Unprocessable entity: {0}")] ValidationError(String),

    #[error("Too many requests")]
    RateLimited,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Onboarding not completed")]
    OnboardingRequired,

    #[error("Internal error: {0}")] InternalError(#[from] anyhow::Error),

    #[error("Database error: {0}")] DatabaseError(#[from] sqlx::Error),

    #[error("Encryption error: {0}")] EncryptionError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::ValidationError(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg.clone())
            }
            AppError::RateLimited =>
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "rate_limited",
                    "Too many requests. Please try again later.".into(),
                ),
            AppError::EmailNotVerified =>
                (
                    StatusCode::FORBIDDEN,
                    "email_not_verified",
                    "Please verify your email address first.".into(),
                ),
            AppError::OnboardingRequired =>
                (
                    StatusCode::FORBIDDEN,
                    "onboarding_required",
                    "Please complete the onboarding assessment first.".into(),
                ),
            AppError::InternalError(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred.".into(),
                )
            }
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred.".into(),
                )
            }
            AppError::EncryptionError(msg) => {
                tracing::error!("Encryption error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "encryption_error",
                    "An encryption error occurred.".into(),
                )
            }
        };

        let body =
            json!({
            "success": false,
            "error": {
                "type": error_type,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}
