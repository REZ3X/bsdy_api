//! Service-level integration tests for Mood, Note, Onboarding, Analytics, and Report services.
//!
//! These tests require a running MariaDB. Run with:
//!   cargo test test_services -- --ignored

mod common;

use bsdy_api::crypto::CryptoService;
use bsdy_api::db;
use bsdy_api::models::mental::{ BaselineAssessmentRequest, CreateMoodEntryRequest };
use bsdy_api::models::note::{ CreateNoteRequest, UpdateNoteRequest };
use bsdy_api::models::chat::CreateChatRequest;
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
