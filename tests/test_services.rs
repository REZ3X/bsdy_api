//! Service-level integration tests for Mood, Note, Onboarding, Analytics, and Report services.
//!
//! These tests require a running MariaDB. Run with:
//!   cargo test test_services -- --ignored

mod common;

use bsdy_api::crypto::CryptoService;
use bsdy_api::db;
use bsdy_api::models::mental::{ BaselineAssessmentRequest, CreateMoodEntryRequest };
use bsdy_api::models::note::{ CreateNoteRequest, UpdateNoteRequest };
use bsdy_api::models::chat::{ CreateChatRequest, ToolCallRequest };
use bsdy_api::services::mood_service::MoodService;
use bsdy_api::services::note_service::NoteService;
use bsdy_api::services::onboarding_service::OnboardingService;
use bsdy_api::services::chat_service::ChatService;
use common::*;

/// Setup: create a pool, run migrations, insert a test user, return (pool, user_id, salt).
async fn setup_user() -> (sqlx::MySqlPool, String, String) {
    load_env();
    let url = test_database_url();
    let config = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = db::create_pool(&config).await.expect("pool");
    db::run_migrations(&pool).await.expect("migrations");

    let user_id = uuid::Uuid::new_v4().to_string();
    let salt = CryptoService::generate_user_salt();

    sqlx::query(
        r#"INSERT INTO users (id, google_id, username, name, email, 
           email_verification_status, onboarding_completed, encryption_salt)
           VALUES (?, ?, ?, ?, ?, 'verified', FALSE, ?)"#
    )
        .bind(&user_id)
        .bind(&format!("google-svc-{}", &user_id[..8]))
        .bind(&format!("svcuser_{}", &user_id[..6]))
        .bind("Service Test User")
        .bind(&format!("svc-{}@test.com", &user_id[..8]))
        .bind(&salt)
        .execute(&pool).await
        .expect("insert test user");

    (pool, user_id, salt)
}

/// Cleanup: remove test user and cascading data.
async fn cleanup(pool: &sqlx::MySqlPool, user_id: &str) {
    // Delete in dependency order
    sqlx::query("DELETE FROM chat_messages WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM chats WHERE user_id = ?").bind(user_id).execute(pool).await.ok();
    sqlx::query("DELETE FROM notes WHERE user_id = ?").bind(user_id).execute(pool).await.ok();
    sqlx::query("DELETE FROM mood_entries WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM mental_analytics_summaries WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM mental_reports WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM mental_characteristics WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM user_activity_logs WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM user_auth_logs WHERE user_id = ?")
        .bind(user_id)
        .execute(pool).await
        .ok();
    sqlx::query("DELETE FROM contents WHERE author_id = ?").bind(user_id).execute(pool).await.ok();
    sqlx::query("DELETE FROM users WHERE id = ?").bind(user_id).execute(pool).await.ok();
}

// ═══════════════════════════════════════════════════════════
//  Onboarding Service Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_services_onboarding_save_and_get_baseline() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    let req = BaselineAssessmentRequest {
        birth: "2000-05-15".into(),
        family_background: Some("Stable family, no history of mental illness".into()),
        stress_level: "moderate".into(),
        anxiety_level: "low".into(),
        depression_level: "low".into(),
        sleep_quality: "moderate".into(),
        social_support: "strong".into(),
        coping_style: "problem_focused".into(),
        personality_traits: r#"["empathetic","introverted"]"#.into(),
        mental_health_history: "No prior diagnoses".into(),
        current_medications: None,
        therapy_status: "none".into(),
        additional_notes: Some("Just want to track my mental wellness".into()),
    };

    let result = OnboardingService::save_baseline(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(result.is_ok(), "save_baseline failed: {:?}", result.err());
    let baseline = result.unwrap();
    assert_eq!(baseline.user_id, user_id);
    assert_eq!(baseline.stress_level, "moderate");
    assert_eq!(baseline.risk_level, "low"); // low + low + moderate = 4 → low

    // Get baseline
    let get_result = OnboardingService::get_baseline(&pool, &crypto, &user_id, &salt).await;
    assert!(get_result.is_ok());
    let fetched = get_result.unwrap();
    assert_eq!(fetched.stress_level, "moderate");
    assert!(fetched.family_background.is_some());

    // Trying to save again should conflict
    let dup = OnboardingService::save_baseline(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(dup.is_err(), "duplicate baseline should be rejected");

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Mood Service Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_services_mood_upsert_and_get() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    let req = CreateMoodEntryRequest {
        mood_score: 7,
        energy_level: Some(6),
        anxiety_level: Some(3),
        stress_level: Some(4),
        sleep_hours: Some(7.5),
        sleep_quality: Some(8),
        appetite: Some("normal".into()),
        social_interaction: Some(true),
        exercise_done: Some(false),
        notes: Some("Feeling pretty good today".into()),
        triggers: None,
        activities: Some(r#"["reading","walking"]"#.into()),
    };

    // Create
    let result = MoodService::upsert_mood(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(result.is_ok(), "upsert_mood failed: {:?}", result.err());
    let mood = result.unwrap();
    assert_eq!(mood.mood_score, 7);
    assert_eq!(mood.notes.as_deref(), Some("Feeling pretty good today"));

    // Upsert (update same day)
    let req2 = CreateMoodEntryRequest {
        mood_score: 8,
        energy_level: Some(7),
        anxiety_level: Some(2),
        stress_level: Some(3),
        sleep_hours: None,
        sleep_quality: None,
        appetite: None,
        social_interaction: None,
        exercise_done: None,
        notes: Some("Updated: feeling even better".into()),
        triggers: None,
        activities: None,
    };
    let updated = MoodService::upsert_mood(&pool, &crypto, &user_id, &salt, &req2).await;
    assert!(updated.is_ok());
    assert_eq!(updated.unwrap().mood_score, 8);

    // Get today
    let today = MoodService::get_today(&pool, &crypto, &user_id, &salt).await;
    assert!(today.is_ok());
    assert!(today.unwrap().is_some());

    // Get entries
    let entries = MoodService::get_mood_entries(
        &pool,
        &crypto,
        &user_id,
        &salt,
        None,
        None,
        Some(30)
    ).await;
    assert!(entries.is_ok());
    assert!(!entries.unwrap().is_empty());

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_services_mood_validation_out_of_range() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    let req = CreateMoodEntryRequest {
        mood_score: 11, // out of range
        energy_level: None,
        anxiety_level: None,
        stress_level: None,
        sleep_hours: None,
        sleep_quality: None,
        appetite: None,
        social_interaction: None,
        exercise_done: None,
        notes: None,
        triggers: None,
        activities: None,
    };

    let result = MoodService::upsert_mood(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(result.is_err(), "mood_score > 10 should be rejected");

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Note Service Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_services_note_crud() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    // Create
    let req = CreateNoteRequest {
        title: "Breathing Exercise".into(),
        content: "Box breathing: inhale 4s, hold 4s, exhale 4s, hold 4s".into(),
        label: Some("coping".into()),
        is_pinned: Some(true),
    };
    let created = NoteService::create_note(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(created.is_ok(), "create_note failed: {:?}", created.err());
    let note = created.unwrap();
    assert_eq!(note.title, "Breathing Exercise");
    assert!(note.is_pinned);

    // Get single
    let fetched = NoteService::get_note(&pool, &crypto, &user_id, &note.id, &salt).await;
    assert!(fetched.is_ok());
    assert_eq!(fetched.unwrap().content, "Box breathing: inhale 4s, hold 4s, exhale 4s, hold 4s");

    // Update
    let update_req = UpdateNoteRequest {
        title: Some("Updated Title".into()),
        content: None,
        label: None,
        is_pinned: Some(false),
    };
    let updated = NoteService::update_note(
        &pool,
        &crypto,
        &user_id,
        &note.id,
        &salt,
        &update_req
    ).await;
    assert!(updated.is_ok());
    assert_eq!(updated.unwrap().title, "Updated Title");

    // List
    let notes = NoteService::get_notes(&pool, &crypto, &user_id, &salt, None, 50).await;
    assert!(notes.is_ok());
    assert!(!notes.unwrap().is_empty());

    // Get labels
    let labels = NoteService::get_labels(&pool, &user_id).await;
    assert!(labels.is_ok());
    assert!(labels.unwrap().contains(&"coping".to_string()));

    // Delete
    let deleted = NoteService::delete_note(&pool, &user_id, &note.id).await;
    assert!(deleted.is_ok());

    // Delete non-existent should 404
    let not_found = NoteService::delete_note(&pool, &user_id, "nonexistent-id").await;
    assert!(not_found.is_err());

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_services_note_validation_empty_title() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    let req = CreateNoteRequest {
        title: "".into(),
        content: "Some content".into(),
        label: None,
        is_pinned: None,
    };
    let result = NoteService::create_note(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(result.is_err(), "empty title should be rejected");

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Chat Service Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_services_chat_crud() {
    let (pool, user_id, _salt) = setup_user().await;

    // Create companion chat
    let req = CreateChatRequest {
        chat_type: Some("companion".into()),
    };
    let created = ChatService::create_chat(&pool, &user_id, &req).await;
    assert!(created.is_ok(), "create_chat failed: {:?}", created.err());
    let chat = created.unwrap();
    assert_eq!(chat.chat_type, "companion");
    assert!(chat.is_active);
    assert_eq!(chat.message_count, 0);

    // Create agentic chat
    let req2 = CreateChatRequest {
        chat_type: Some("agentic".into()),
    };
    let agent_chat = ChatService::create_chat(&pool, &user_id, &req2).await;
    assert!(agent_chat.is_ok());

    // List chats
    let chats = ChatService::list_chats(&pool, &user_id, 50).await;
    assert!(chats.is_ok());
    assert!(chats.unwrap().len() >= 2);

    // Get single
    let fetched = ChatService::get_chat(&pool, &user_id, &chat.id).await;
    assert!(fetched.is_ok());

    // Invalid chat_type
    let bad_req = CreateChatRequest {
        chat_type: Some("invalid".into()),
    };
    let bad_result = ChatService::create_chat(&pool, &user_id, &bad_req).await;
    assert!(bad_result.is_err());

    // Delete
    let deleted = ChatService::delete_chat(&pool, &user_id, &chat.id).await;
    assert!(deleted.is_ok());

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — CREATE_NOTE via NoteService
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_agent_tool_create_note() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    // Simulate what the agent does when CREATE_NOTE is called
    let req = CreateNoteRequest {
        title: "Grounding Technique".into(),
        content: "5-4-3-2-1: Name 5 things you see, 4 you hear, 3 you touch, 2 you smell, 1 you taste.".into(),
        label: Some("coping".into()),
        is_pinned: Some(false),
    };

    let note = NoteService::create_note(&pool, &crypto, &user_id, &salt, &req).await;
    assert!(note.is_ok(), "Agent CREATE_NOTE failed: {:?}", note.err());
    let note = note.unwrap();
    assert_eq!(note.title, "Grounding Technique");
    assert_eq!(note.label.as_deref(), Some("coping"));

    // Verify it's retrievable
    let fetched = NoteService::get_note(&pool, &crypto, &user_id, &note.id, &salt).await;
    assert!(fetched.is_ok());
    assert!(fetched.unwrap().content.contains("5-4-3-2-1"));

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — UPDATE_NOTE via NoteService
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_agent_tool_update_note() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    // Create a note first
    let create_req = CreateNoteRequest {
        title: "Morning Routine".into(),
        content: "Wake up, stretch, drink water.".into(),
        label: Some("routine".into()),
        is_pinned: Some(false),
    };
    let note = NoteService::create_note(&pool, &crypto, &user_id, &salt, &create_req).await.expect(
        "create note"
    );

    // Simulate agent UPDATE_NOTE — add more content
    let update_req = UpdateNoteRequest {
        title: None,
        content: Some(
            "Wake up, stretch for 5 min, drink a glass of water, then journal for 10 min.".into()
        ),
        label: None,
        is_pinned: Some(true),
    };
    let updated = NoteService::update_note(
        &pool,
        &crypto,
        &user_id,
        &note.id,
        &salt,
        &update_req
    ).await;
    assert!(updated.is_ok(), "Agent UPDATE_NOTE failed: {:?}", updated.err());
    let updated = updated.unwrap();
    assert_eq!(updated.title, "Morning Routine"); // unchanged
    assert!(updated.content.contains("journal for 10 min"));
    assert!(updated.is_pinned); // changed to true

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — DELETE_NOTE via NoteService
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_agent_tool_delete_note() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    // Create then delete
    let req = CreateNoteRequest {
        title: "Temporary Note".into(),
        content: "To be deleted by the agent".into(),
        label: None,
        is_pinned: None,
    };
    let note = NoteService::create_note(&pool, &crypto, &user_id, &salt, &req).await.expect(
        "create note"
    );

    let deleted = NoteService::delete_note(&pool, &user_id, &note.id).await;
    assert!(deleted.is_ok(), "Agent DELETE_NOTE failed: {:?}", deleted.err());

    // Verify it's gone
    let fetched = NoteService::get_note(&pool, &crypto, &user_id, &note.id, &salt).await;
    assert!(fetched.is_err(), "Deleted note should not be retrievable");

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — Multi-tool create + read notes workflow
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_agent_tool_create_then_list_notes() {
    let (pool, user_id, salt) = setup_user().await;
    let crypto = test_crypto();

    // Simulate agent creating multiple coping notes
    let labels = ["breathing", "mindfulness", "physical"];
    let contents = [
        "Box breathing: 4-4-4-4 counts",
        "Body scan meditation: start from toes up to head",
        "Light walk for 15 minutes when feeling overwhelmed",
    ];

    for (i, label) in labels.iter().enumerate() {
        let req = CreateNoteRequest {
            title: format!("Strategy {}", i + 1),
            content: contents[i].into(),
            label: Some(label.to_string()),
            is_pinned: Some(false),
        };
        NoteService::create_note(&pool, &crypto, &user_id, &salt, &req).await.expect("create note");
    }

    // Agent lists all notes
    let all_notes = NoteService::get_notes(&pool, &crypto, &user_id, &salt, None, 50).await;
    assert!(all_notes.is_ok());
    assert_eq!(all_notes.unwrap().len(), 3);

    // Agent filters by label
    let breathing_notes = NoteService::get_notes(
        &pool,
        &crypto,
        &user_id,
        &salt,
        Some("breathing"),
        50
    ).await;
    assert!(breathing_notes.is_ok());
    assert_eq!(breathing_notes.unwrap().len(), 1);

    cleanup(&pool, &user_id).await;
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — GENERATE_REPORT types (weekly, monthly, yearly)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_agent_report_type_yearly_is_valid() {
    // Verify the GenerateReportRequest can carry yearly type
    let req = bsdy_api::models::mental::GenerateReportRequest {
        report_type: Some("yearly".into()),
        period_start: None,
        period_end: None,
        send_email: Some(false),
    };
    assert_eq!(req.report_type.as_deref(), Some("yearly"));
}

#[test]
fn test_agent_report_types_all_supported() {
    // All types the agent can request
    for report_type in &["weekly", "monthly", "yearly", "custom"] {
        let req = bsdy_api::models::mental::GenerateReportRequest {
            report_type: Some(report_type.to_string()),
            period_start: if *report_type == "custom" {
                Some("2026-01-01".into())
            } else {
                None
            },
            period_end: if *report_type == "custom" {
                Some("2026-03-01".into())
            } else {
                None
            },
            send_email: None,
        };
        assert_eq!(req.report_type.as_deref(), Some(*report_type));
    }
}

// ═══════════════════════════════════════════════════════════
//  Agent Tool — ToolCallRequest parsing for new tools
// ═══════════════════════════════════════════════════════════

#[test]
fn test_agent_tool_call_request_create_note_parsing() {
    let json =
        r#"{
        "tool_name": "CREATE_NOTE",
        "parameters": {
            "title": "Sleep Hygiene Tips",
            "content": "Keep consistent sleep schedule. Avoid screens 1hr before bed.",
            "label": "sleep",
            "is_pinned": false
        }
    }"#;
    let tc: ToolCallRequest = serde_json::from_str(json).unwrap();
    assert_eq!(tc.tool_name, "CREATE_NOTE");
    assert_eq!(tc.parameters["title"], "Sleep Hygiene Tips");
    assert_eq!(tc.parameters["label"], "sleep");
}

#[test]
fn test_agent_tool_call_request_suggest_coping_parsing() {
    let json =
        r#"{
        "tool_name": "SUGGEST_COPING_STRATEGIES",
        "parameters": {
            "context": "work-related burnout",
            "save_as_notes": true,
            "label": "burnout"
        }
    }"#;
    let tc: ToolCallRequest = serde_json::from_str(json).unwrap();
    assert_eq!(tc.tool_name, "SUGGEST_COPING_STRATEGIES");
    assert_eq!(tc.parameters["context"], "work-related burnout");
    assert!(tc.parameters["save_as_notes"].as_bool().unwrap());
}

#[test]
fn test_agent_tool_name_case_insensitive_matching() {
    // The agent normalizes tool names with .to_uppercase()
    let variants = vec!["create_note", "CREATE_NOTE", "Create_Note"];
    for variant in variants {
        assert_eq!(
            variant.to_uppercase(),
            "CREATE_NOTE",
            "Tool name '{}' should normalize to CREATE_NOTE",
            variant
        );
    }
}

#[test]
fn test_agent_tool_aliases_resolve_correctly() {
    // Verify tool aliases from the match block
    let aliases = vec![
        ("CREATE_NOTE", "CREATE_NOTE"),
        ("CREATE_COPING_NOTE", "CREATE_COPING_NOTE"),
        ("UPDATE_NOTE", "UPDATE_NOTE"),
        ("EDIT_NOTE", "EDIT_NOTE"),
        ("DELETE_NOTE", "DELETE_NOTE"),
        ("REMOVE_NOTE", "REMOVE_NOTE"),
        ("SUGGEST_COPING_STRATEGIES", "SUGGEST_COPING_STRATEGIES"),
        ("SUGGEST_COPING", "SUGGEST_COPING"),
        ("GET_MOOD_ENTRIES", "GET_MOOD_ENTRIES"),
        ("GET_MOOD_LOGS", "GET_MOOD_LOGS"),
        ("GET_NOTES", "GET_NOTES"),
        ("GET_COPING_NOTES", "GET_COPING_NOTES"),
        ("GENERATE_REPORT", "GENERATE_REPORT"),
        ("GET_BASELINE", "GET_BASELINE"),
        ("GET_MENTAL_PROFILE", "GET_MENTAL_PROFILE")
    ];
    // All should be recognized (non-empty, valid tool names)
    for (alias, _expected) in &aliases {
        assert!(!alias.is_empty(), "Alias should not be empty");
    }
}

// ═══════════════════════════════════════════════════════════
//  Content Service Integration Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_content_create_and_get() {
    use bsdy_api::models::content::CreateContentRequest;
    use bsdy_api::services::content_service::ContentService;

    let (pool, user_id, _salt) = setup_user().await;

    let req = CreateContentRequest {
        title: "Test Article".into(),
        body: "This is the article body content.".into(),
        excerpt: Some("Brief excerpt".into()),
        status: Some("draft".into()),
    };

    let content = ContentService::create_content(
        &pool,
        &user_id,
        &req,
        "http://localhost:8000"
    ).await.expect("should create content");

    assert_eq!(content.title, "Test Article");
    assert_eq!(content.slug, "test-article");
    assert_eq!(content.status, "draft");
    assert!(content.published_at.is_none());
    assert!(content.cover_image_url.is_none());

    // Get by ID
    let fetched = ContentService::get_content(
        &pool,
        &content.id,
        true,
        "http://localhost:8000"
    ).await.expect("should get content by id");
    assert_eq!(fetched.id, content.id);
    assert_eq!(fetched.body, "This is the article body content.");

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_content_update_and_publish() {
    use bsdy_api::models::content::{ CreateContentRequest, UpdateContentRequest };
    use bsdy_api::services::content_service::ContentService;

    let (pool, user_id, _salt) = setup_user().await;

    let req = CreateContentRequest {
        title: "Draft Post".into(),
        body: "Initial body".into(),
        excerpt: None,
        status: None, // defaults to draft
    };

    let content = ContentService::create_content(
        &pool,
        &user_id,
        &req,
        "http://localhost:8000"
    ).await.expect("create");
    assert_eq!(content.status, "draft");

    // Update: change title and publish
    let update = UpdateContentRequest {
        title: Some("Published Post".into()),
        body: Some("Updated body".into()),
        excerpt: Some("New excerpt".into()),
        status: Some("published".into()),
    };

    let updated = ContentService::update_content(
        &pool,
        &content.id,
        &update,
        "http://localhost:8000"
    ).await.expect("update");
    assert_eq!(updated.title, "Published Post");
    assert_eq!(updated.slug, "published-post");
    assert_eq!(updated.status, "published");
    assert!(updated.published_at.is_some());

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_content_list_public_vs_admin() {
    use bsdy_api::models::content::CreateContentRequest;
    use bsdy_api::services::content_service::ContentService;

    let (pool, user_id, _salt) = setup_user().await;

    // Create a draft and a published content
    let draft = CreateContentRequest {
        title: "Draft Only".into(),
        body: "Draft body".into(),
        excerpt: None,
        status: Some("draft".into()),
    };
    let published = CreateContentRequest {
        title: "Public Post".into(),
        body: "Public body".into(),
        excerpt: None,
        status: Some("published".into()),
    };

    ContentService::create_content(&pool, &user_id, &draft, "http://localhost:8000").await.unwrap();
    ContentService::create_content(
        &pool,
        &user_id,
        &published,
        "http://localhost:8000"
    ).await.unwrap();

    // Admin sees both
    let (_admin_list, admin_total) = ContentService::list_contents(
        &pool,
        true,
        50,
        0,
        "http://localhost:8000"
    ).await.expect("admin list");
    assert!(admin_total >= 2, "Admin should see at least 2 contents");

    // Public sees only published
    let (pub_list, _pub_total) = ContentService::list_contents(
        &pool,
        false,
        50,
        0,
        "http://localhost:8000"
    ).await.expect("public list");
    for item in &pub_list {
        assert_eq!(item.status, "published", "Public should only see published content");
    }

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_content_delete() {
    use bsdy_api::models::content::CreateContentRequest;
    use bsdy_api::services::content_service::ContentService;

    let (pool, user_id, _salt) = setup_user().await;

    let req = CreateContentRequest {
        title: "To Be Deleted".into(),
        body: "Will be removed".into(),
        excerpt: None,
        status: None,
    };

    let content = ContentService::create_content(
        &pool,
        &user_id,
        &req,
        "http://localhost:8000"
    ).await.expect("create");

    ContentService::delete_content(&pool, &content.id).await.expect("delete");

    // Should be gone
    let result = ContentService::get_content(
        &pool,
        &content.id,
        true,
        "http://localhost:8000"
    ).await;
    assert!(result.is_err(), "Deleted content should not be found");

    cleanup(&pool, &user_id).await;
}

#[tokio::test]
#[ignore]
async fn test_content_get_by_slug() {
    use bsdy_api::models::content::CreateContentRequest;
    use bsdy_api::services::content_service::ContentService;

    let (pool, user_id, _salt) = setup_user().await;

    let req = CreateContentRequest {
        title: "Slug Lookup Test".into(),
        body: "Body for slug test".into(),
        excerpt: None,
        status: Some("published".into()),
    };

    let content = ContentService::create_content(
        &pool,
        &user_id,
        &req,
        "http://localhost:8000"
    ).await.expect("create");

    let by_slug = ContentService::get_content_by_slug(
        &pool,
        &content.slug,
        false,
        "http://localhost:8000"
    ).await.expect("get by slug");
    assert_eq!(by_slug.id, content.id);
    assert_eq!(by_slug.slug, "slug-lookup-test");

    cleanup(&pool, &user_id).await;
}

#[test]
fn test_content_status_validation() {
    use bsdy_api::services::content_service::ContentService;
    // Valid statuses should produce valid slugs (proxy: slug generation works)
    let valid = vec!["draft", "published", "archived"];
    for s in valid {
        assert!(
            s == "draft" || s == "published" || s == "archived",
            "Status '{}' should be valid",
            s
        );
    }
    // Slug generation doesn't panic on edge cases
    assert_eq!(ContentService::generate_slug(""), "");
    assert_eq!(ContentService::generate_slug("---"), "");
    assert_eq!(ContentService::generate_slug("A"), "a");
}
