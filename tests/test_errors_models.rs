//! Tests for AppError responses and model DTOs.

use axum::response::IntoResponse;
use bsdy_api::error::AppError;
use bsdy_api::models::user::*;
use bsdy_api::models::chat::*;
use bsdy_api::models::note::*;
use chrono::NaiveDateTime;

// ═══════════════════════════════════════════════════════════
//  AppError → HTTP Response Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_error_bad_request_status() {
    let err = AppError::BadRequest("invalid input".into());
    let response = err.into_response();
    assert_eq!(response.status(), 400);
}

#[test]
fn test_error_unauthorized_status() {
    let err = AppError::Unauthorized("no token".into());
    let response = err.into_response();
    assert_eq!(response.status(), 401);
}

#[test]
fn test_error_forbidden_status() {
    let err = AppError::Forbidden("access denied".into());
    let response = err.into_response();
    assert_eq!(response.status(), 403);
}

#[test]
fn test_error_not_found_status() {
    let err = AppError::NotFound("resource missing".into());
    let response = err.into_response();
    assert_eq!(response.status(), 404);
}

#[test]
fn test_error_conflict_status() {
    let err = AppError::Conflict("already exists".into());
    let response = err.into_response();
    assert_eq!(response.status(), 409);
}

#[test]
fn test_error_validation_status() {
    let err = AppError::ValidationError("field required".into());
    let response = err.into_response();
    assert_eq!(response.status(), 422);
}

#[test]
fn test_error_rate_limited_status() {
    let err = AppError::RateLimited;
    let response = err.into_response();
    assert_eq!(response.status(), 429);
}

#[test]
fn test_error_email_not_verified_status() {
    let err = AppError::EmailNotVerified;
    let response = err.into_response();
    assert_eq!(response.status(), 403);
}

#[test]
fn test_error_onboarding_required_status() {
    let err = AppError::OnboardingRequired;
    let response = err.into_response();
    assert_eq!(response.status(), 403);
}

#[test]
fn test_error_encryption_status() {
    let err = AppError::EncryptionError("bad key".into());
    let response = err.into_response();
    assert_eq!(response.status(), 500);
}

#[test]
fn test_error_internal_status() {
    let err = AppError::InternalError(anyhow::anyhow!("something broke"));
    let response = err.into_response();
    assert_eq!(response.status(), 500);
}

// ═══════════════════════════════════════════════════════════
//  Model DTO Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_user_row_to_response() {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let user = UserRow {
        id: "u-123".into(),
        google_id: "g-123".into(),
        username: "testuser".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        avatar_url: Some("https://img.example.com/avatar.png".into()),
        birth: None,
        email_verification_status: "verified".into(),
        email_verification_token: None,
        email_verified_at: Some(now),
        onboarding_completed: true,
        encryption_salt: "abcd1234".into(),
        created_at: now,
        updated_at: now,
    };

    let resp = UserResponse::from(&user);
    assert_eq!(resp.id, "u-123");
    assert_eq!(resp.username, "testuser");
    assert!(resp.email_verified);
    assert!(resp.onboarding_completed);
    assert!(resp.avatar_url.is_some());
}

#[test]
fn test_user_row_unverified() {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let user = UserRow {
        id: "u-456".into(),
        google_id: "g-456".into(),
        username: "newuser".into(),
        name: "New User".into(),
        email: "new@example.com".into(),
        avatar_url: None,
        birth: None,
        email_verification_status: "pending".into(),
        email_verification_token: Some("token123".into()),
        email_verified_at: None,
        onboarding_completed: false,
        encryption_salt: "salt456".into(),
        created_at: now,
        updated_at: now,
    };

    let resp = UserResponse::from(&user);
    assert!(!resp.email_verified);
    assert!(!resp.onboarding_completed);
    assert!(resp.avatar_url.is_none());
}

#[test]
fn test_chat_row_to_response() {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let chat = ChatRow {
        id: "c-1".into(),
        user_id: "u-1".into(),
        title: "My Chat".into(),
        chat_type: "companion".into(),
        is_active: true,
        message_count: 5,
        created_at: now,
        updated_at: now,
    };

    let resp = ChatResponse::from(&chat);
    assert_eq!(resp.id, "c-1");
    assert_eq!(resp.title, "My Chat");
    assert_eq!(resp.chat_type, "companion");
    assert!(resp.is_active);
    assert_eq!(resp.message_count, 5);
}

#[test]
fn test_create_note_request_deserialization() {
    let json =
        r#"{
        "title": "Breathing Exercise",
        "content": "Box breathing: 4-4-4-4",
        "label": "coping",
        "is_pinned": true
    }"#;
    let req: CreateNoteRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.title, "Breathing Exercise");
    assert_eq!(req.label.as_deref(), Some("coping"));
    assert_eq!(req.is_pinned, Some(true));
}

#[test]
fn test_send_message_request_deserialization() {
    let json = r#"{"message": "I feel anxious today"}"#;
    let req: SendMessageRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.message, "I feel anxious today");
}

#[test]
fn test_tool_call_serialization() {
    let tc = ToolCall {
        tool_name: "GET_MOOD_ENTRIES".into(),
        parameters: serde_json::json!({"period": "weekly"}),
    };
    let json = serde_json::to_string(&tc).unwrap();
    assert!(json.contains("GET_MOOD_ENTRIES"));
    assert!(json.contains("weekly"));
}

#[test]
fn test_tool_result_serialization() {
    let tr = ToolResult {
        tool_name: "GET_MOOD_ENTRIES".into(),
        result: serde_json::json!({"entries": []}),
        success: true,
    };
    let json = serde_json::to_string(&tr).unwrap();
    assert!(json.contains("\"success\":true"));
}
