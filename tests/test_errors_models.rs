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
        role: "basic".into(),
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
        role: "basic".into(),
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

// ═══════════════════════════════════════════════════════════
//  AgentResponse Parsing Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_agent_response_with_tool_calls_deserialization() {
    let json =
        r#"{
        "response": "Let me fetch your mood data",
        "tool_calls": [
            {
                "tool_name": "GET_MOOD_ENTRIES",
                "parameters": { "limit": 7 }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.response, "Let me fetch your mood data");
    assert_eq!(resp.tool_calls.len(), 1);
    assert_eq!(resp.tool_calls[0].tool_name, "GET_MOOD_ENTRIES");
    assert_eq!(resp.tool_calls[0].parameters["limit"], 7);
}

#[test]
fn test_agent_response_without_tool_calls() {
    let json = r#"{
        "response": "I'm here to help!",
        "tool_calls": []
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.response, "I'm here to help!");
    assert!(resp.tool_calls.is_empty());
}

#[test]
fn test_agent_response_plain_text_fallback() {
    // When AI returns no tool_calls field, it defaults to empty vec
    let json = r#"{ "response": "Just chatting" }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.response, "Just chatting");
    assert!(resp.tool_calls.is_empty());
}

#[test]
fn test_agent_response_multiple_tool_calls() {
    let json =
        r#"{
        "response": "Let me check your mood and notes",
        "tool_calls": [
            { "tool_name": "GET_MOOD_ENTRIES", "parameters": { "limit": 14 } },
            { "tool_name": "GET_NOTES", "parameters": { "label": "coping", "limit": 5 } }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls.len(), 2);
    assert_eq!(resp.tool_calls[0].tool_name, "GET_MOOD_ENTRIES");
    assert_eq!(resp.tool_calls[1].tool_name, "GET_NOTES");
    assert_eq!(resp.tool_calls[1].parameters["label"], "coping");
}

#[test]
fn test_agent_response_create_note_tool_call() {
    let json =
        r#"{
        "response": "I'll create a coping note for you",
        "tool_calls": [
            {
                "tool_name": "CREATE_NOTE",
                "parameters": {
                    "title": "Breathing Exercise",
                    "content": "Box breathing: inhale 4s, hold 4s, exhale 4s, hold 4s",
                    "label": "breathing",
                    "is_pinned": true
                }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls.len(), 1);
    assert_eq!(resp.tool_calls[0].tool_name, "CREATE_NOTE");
    assert_eq!(resp.tool_calls[0].parameters["title"], "Breathing Exercise");
    assert_eq!(resp.tool_calls[0].parameters["is_pinned"], true);
}

#[test]
fn test_agent_response_update_note_tool_call() {
    let json =
        r#"{
        "response": "I'll update that note for you",
        "tool_calls": [
            {
                "tool_name": "UPDATE_NOTE",
                "parameters": {
                    "note_id": "abc-123",
                    "content": "Updated content with better instructions",
                    "is_pinned": false
                }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls[0].tool_name, "UPDATE_NOTE");
    assert_eq!(resp.tool_calls[0].parameters["note_id"], "abc-123");
    assert!(resp.tool_calls[0].parameters["content"].as_str().is_some());
}

#[test]
fn test_agent_response_delete_note_tool_call() {
    let json =
        r#"{
        "response": "Removing that note now",
        "tool_calls": [
            {
                "tool_name": "DELETE_NOTE",
                "parameters": { "note_id": "xyz-789" }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls[0].tool_name, "DELETE_NOTE");
    assert_eq!(resp.tool_calls[0].parameters["note_id"], "xyz-789");
}

#[test]
fn test_agent_response_suggest_coping_tool_call() {
    let json =
        r#"{
        "response": "Let me generate some personalized strategies",
        "tool_calls": [
            {
                "tool_name": "SUGGEST_COPING_STRATEGIES",
                "parameters": {
                    "context": "anxiety before exams",
                    "save_as_notes": true,
                    "label": "exam-anxiety"
                }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls[0].tool_name, "SUGGEST_COPING_STRATEGIES");
    assert_eq!(resp.tool_calls[0].parameters["save_as_notes"], true);
    assert_eq!(resp.tool_calls[0].parameters["label"], "exam-anxiety");
}

#[test]
fn test_agent_response_generate_report_tool_call() {
    let json =
        r#"{
        "response": "Generating your yearly report now",
        "tool_calls": [
            {
                "tool_name": "GENERATE_REPORT",
                "parameters": {
                    "report_type": "yearly",
                    "send_email": false
                }
            }
        ]
    }"#;
    let resp: AgentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.tool_calls[0].tool_name, "GENERATE_REPORT");
    assert_eq!(resp.tool_calls[0].parameters["report_type"], "yearly");
    assert_eq!(resp.tool_calls[0].parameters["send_email"], false);
}

#[test]
fn test_agent_response_from_markdown_code_block() {
    // AI sometimes wraps JSON in markdown code blocks
    let raw =
        r#"```json
{
    "response": "Checking your data",
    "tool_calls": [
        { "tool_name": "GET_BASELINE", "parameters": {} }
    ]
}
```"#;
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    let resp: AgentResponse = serde_json::from_str(cleaned).unwrap();
    assert_eq!(resp.tool_calls.len(), 1);
    assert_eq!(resp.tool_calls[0].tool_name, "GET_BASELINE");
}

#[test]
fn test_tool_call_all_new_tools_serialize() {
    // Verify all new tool types can be serialized/deserialized round-trip
    let tools = vec![
        ToolCall {
            tool_name: "CREATE_NOTE".into(),
            parameters: serde_json::json!({"title": "t", "content": "c"}),
        },
        ToolCall {
            tool_name: "UPDATE_NOTE".into(),
            parameters: serde_json::json!({"note_id": "id", "title": "new"}),
        },
        ToolCall {
            tool_name: "DELETE_NOTE".into(),
            parameters: serde_json::json!({"note_id": "id"}),
        },
        ToolCall {
            tool_name: "SUGGEST_COPING_STRATEGIES".into(),
            parameters: serde_json::json!({"context": "stress", "save_as_notes": true}),
        }
    ];
    for tc in &tools {
        let json = serde_json::to_string(tc).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tool_name, tc.tool_name);
    }
}

// ═══════════════════════════════════════════════════════════
//  Content Model Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_content_row_to_response() {
    use bsdy_api::models::content::ContentRow;
    let now = NaiveDateTime::parse_from_str("2026-03-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let row = ContentRow {
        id: "c-123".into(),
        author_id: "u-admin".into(),
        title: "Test Article".into(),
        slug: "test-article".into(),
        body: "Full body content".into(),
        excerpt: Some("Brief excerpt".into()),
        cover_image: Some("img-123.jpg".into()),
        status: "published".into(),
        published_at: Some(now),
        created_at: now,
        updated_at: now,
    };

    let resp = row.to_response("http://localhost:8000");
    assert_eq!(resp.id, "c-123");
    assert_eq!(resp.title, "Test Article");
    assert_eq!(resp.slug, "test-article");
    assert_eq!(resp.body, "Full body content");
    assert_eq!(resp.excerpt.as_deref(), Some("Brief excerpt"));
    assert_eq!(
        resp.cover_image_url.as_deref(),
        Some("http://localhost:8000/uploads/content/img-123.jpg")
    );
    assert_eq!(resp.status, "published");
    assert!(resp.published_at.is_some());
}

#[test]
fn test_content_row_to_list_item() {
    use bsdy_api::models::content::ContentRow;
    let now = NaiveDateTime::parse_from_str("2026-03-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let row = ContentRow {
        id: "c-456".into(),
        author_id: "u-admin".into(),
        title: "Another Article".into(),
        slug: "another-article".into(),
        body: "Body content that should not appear in list item".into(),
        excerpt: None,
        cover_image: None,
        status: "draft".into(),
        published_at: None,
        created_at: now,
        updated_at: now,
    };

    let item = row.to_list_item("http://localhost:8000");
    assert_eq!(item.id, "c-456");
    assert_eq!(item.slug, "another-article");
    assert!(item.cover_image_url.is_none());
    assert!(item.published_at.is_none());
    assert_eq!(item.status, "draft");
}

#[test]
fn test_content_row_no_cover_image_url() {
    use bsdy_api::models::content::ContentRow;
    let now = NaiveDateTime::parse_from_str("2026-03-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let row = ContentRow {
        id: "c-789".into(),
        author_id: "u-admin".into(),
        title: "No Image".into(),
        slug: "no-image".into(),
        body: "body".into(),
        excerpt: None,
        cover_image: None,
        status: "published".into(),
        published_at: Some(now),
        created_at: now,
        updated_at: now,
    };

    let resp = row.to_response("http://example.com");
    assert!(resp.cover_image_url.is_none());
}

#[test]
fn test_create_content_request_deserialize() {
    use bsdy_api::models::content::CreateContentRequest;
    let json =
        r#"{"title": "Hello World", "body": "Content body", "excerpt": "Short", "status": "published"}"#;
    let req: CreateContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.title, "Hello World");
    assert_eq!(req.body, "Content body");
    assert_eq!(req.excerpt.as_deref(), Some("Short"));
    assert_eq!(req.status.as_deref(), Some("published"));
}

#[test]
fn test_create_content_request_minimal() {
    use bsdy_api::models::content::CreateContentRequest;
    let json = r#"{"title": "Minimal", "body": "Just title and body"}"#;
    let req: CreateContentRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.title, "Minimal");
    assert!(req.excerpt.is_none());
    assert!(req.status.is_none());
}

#[test]
fn test_update_content_request_all_optional() {
    use bsdy_api::models::content::UpdateContentRequest;
    let json = r#"{}"#;
    let req: UpdateContentRequest = serde_json::from_str(json).unwrap();
    assert!(req.title.is_none());
    assert!(req.body.is_none());
    assert!(req.excerpt.is_none());
    assert!(req.status.is_none());
}

// ═══════════════════════════════════════════════════════════
//  User Role Model Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_user_response_includes_role() {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let user = UserRow {
        id: "u-admin".into(),
        google_id: "g-admin".into(),
        username: "adminuser".into(),
        name: "Admin User".into(),
        email: "admin@example.com".into(),
        avatar_url: None,
        birth: None,
        email_verification_status: "verified".into(),
        email_verification_token: None,
        email_verified_at: Some(now),
        onboarding_completed: true,
        role: "admin".into(),
        encryption_salt: "salt1234".into(),
        created_at: now,
        updated_at: now,
    };

    let resp = UserResponse::from(&user);
    assert_eq!(resp.role, "admin");
}

#[test]
fn test_user_response_basic_role_default() {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let user = UserRow {
        id: "u-basic".into(),
        google_id: "g-basic".into(),
        username: "basicuser".into(),
        name: "Basic User".into(),
        email: "basic@example.com".into(),
        avatar_url: None,
        birth: None,
        email_verification_status: "verified".into(),
        email_verification_token: None,
        email_verified_at: Some(now),
        onboarding_completed: false,
        role: "basic".into(),
        encryption_salt: "salt5678".into(),
        created_at: now,
        updated_at: now,
    };

    let resp = UserResponse::from(&user);
    assert_eq!(resp.role, "basic");
}
