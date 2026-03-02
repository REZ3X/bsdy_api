//! Database integration tests.
//!
//! These tests require a running MariaDB instance. They are #[ignore]'d by default.
//! Set TEST_DATABASE_URL (or DATABASE_URL) env var and run:
//!   cargo test test_db -- --ignored
//!
//! Recommended: create a `bsdy_test` database for isolated testing.

mod common;

use bsdy_api::config::DatabaseConfig;
use bsdy_api::db;
use common::*;

/// Helper: create a real pool from env.
async fn test_pool() -> sqlx::MySqlPool {
    load_env();
    let url = test_database_url();
    let config = DatabaseConfig {
        url,
        max_connections: 2,
    };
    db::create_pool(&config).await.expect("Failed to create test DB pool")
}

// ═══════════════════════════════════════════════════════════
//  Database Connection Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_pool_creation() {
    let pool = test_pool().await;
    // Verify we can execute a simple query
    let row: (i32,) = sqlx
        ::query_as("SELECT 1")
        .fetch_one(&pool).await
        .expect("SELECT 1 should work");
    assert_eq!(row.0, 1);
}

#[tokio::test]
#[ignore]
async fn test_db_migrations_run_successfully() {
    let pool = test_pool().await;
    let result = db::run_migrations(&pool).await;
    assert!(result.is_ok(), "Migrations failed: {:?}", result.err());
}

#[tokio::test]
#[ignore]
async fn test_db_tables_exist_after_migration() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.expect("migrations");

    // Check that core tables exist
    let tables = [
        "users",
        "mental_characteristics",
        "mood_entries",
        "mental_analytics_summaries",
        "mental_reports",
        "chats",
        "chat_messages",
        "notes",
        "user_auth_logs",
        "user_activity_logs",
        "scheduled_tasks",
    ];

    for table in &tables {
        let result: Result<(i64,), _> = sqlx
            ::query_as(&format!("SELECT COUNT(*) FROM {}", table))
            .fetch_one(&pool).await;
        assert!(
            result.is_ok(),
            "Table '{}' should exist after migration, got: {:?}",
            table,
            result.err()
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  User CRUD Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_user_insert_and_select() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let salt = bsdy_api::crypto::CryptoService::generate_user_salt();

    // Insert
    sqlx::query(
        r#"INSERT INTO users (id, google_id, username, name, email, email_verification_status,
           onboarding_completed, encryption_salt)
           VALUES (?, ?, ?, ?, ?, 'pending', FALSE, ?)"#
    )
        .bind(&user_id)
        .bind("google-test-1")
        .bind("testuser_db")
        .bind("DB Test User")
        .bind(&format!("dbtest-{}@test.com", &user_id[..8]))
        .bind(&salt)
        .execute(&pool).await
        .expect("INSERT user");

    // Select
    let row: (String, String, String) = sqlx
        ::query_as("SELECT id, name, email FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&pool).await
        .expect("SELECT user");

    assert_eq!(row.0, user_id);
    assert_eq!(row.1, "DB Test User");

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = ?").bind(&user_id).execute(&pool).await.ok();
}

// ═══════════════════════════════════════════════════════════
//  Mood Entry CRUD Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_mood_entry_insert_and_query() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let salt = bsdy_api::crypto::CryptoService::generate_user_salt();
    let mood_id = uuid::Uuid::new_v4().to_string();

    // Insert user first (FK constraint)
    sqlx::query(
        r#"INSERT INTO users (id, google_id, username, name, email, email_verification_status,
           onboarding_completed, encryption_salt)
           VALUES (?, ?, ?, ?, ?, 'verified', TRUE, ?)"#
    )
        .bind(&user_id)
        .bind("google-mood-1")
        .bind(&format!("mooduser_{}", &user_id[..6]))
        .bind("Mood Test User")
        .bind(&format!("mood-{}@test.com", &user_id[..8]))
        .bind(&salt)
        .execute(&pool).await
        .unwrap();

    // Insert mood entry
    sqlx::query(
        r#"INSERT INTO mood_entries (id, user_id, entry_date, mood_score, energy_level, anxiety_level, stress_level)
           VALUES (?, ?, CURDATE(), 7, 6, 3, 4)"#
    )
        .bind(&mood_id)
        .bind(&user_id)
        .execute(&pool).await
        .expect("INSERT mood entry");

    // Query
    let row: (String, i8) = sqlx
        ::query_as("SELECT id, mood_score FROM mood_entries WHERE id = ?")
        .bind(&mood_id)
        .fetch_one(&pool).await
        .expect("SELECT mood entry");

    assert_eq!(row.0, mood_id);
    assert_eq!(row.1, 7);

    // Cleanup
    sqlx::query("DELETE FROM mood_entries WHERE id = ?").bind(&mood_id).execute(&pool).await.ok();
    sqlx::query("DELETE FROM users WHERE id = ?").bind(&user_id).execute(&pool).await.ok();
}

// ═══════════════════════════════════════════════════════════
//  Note CRUD Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_note_insert_update_delete() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let note_id = uuid::Uuid::new_v4().to_string();
    let salt = bsdy_api::crypto::CryptoService::generate_user_salt();
    let crypto = test_crypto();

    // Insert user
    sqlx::query(
        r#"INSERT INTO users (id, google_id, username, name, email, email_verification_status,
           onboarding_completed, encryption_salt)
           VALUES (?, ?, ?, ?, ?, 'verified', TRUE, ?)"#
    )
        .bind(&user_id)
        .bind("google-note-1")
        .bind(&format!("noteuser_{}", &user_id[..6]))
        .bind("Note User")
        .bind(&format!("note-{}@test.com", &user_id[..8]))
        .bind(&salt)
        .execute(&pool).await
        .unwrap();

    // Insert encrypted note
    let title_enc = crypto.encrypt("Test Note Title", &salt).unwrap();
    let content_enc = crypto.encrypt("This is my coping strategy", &salt).unwrap();

    sqlx::query(
        r#"INSERT INTO notes (id, user_id, title_enc, content_enc, label, is_pinned)
           VALUES (?, ?, ?, ?, 'coping', FALSE)"#
    )
        .bind(&note_id)
        .bind(&user_id)
        .bind(&title_enc)
        .bind(&content_enc)
        .execute(&pool).await
        .expect("INSERT note");

    // Verify we can read and decrypt
    let row: (String, String) = sqlx
        ::query_as("SELECT title_enc, content_enc FROM notes WHERE id = ?")
        .bind(&note_id)
        .fetch_one(&pool).await
        .expect("SELECT note");

    let decrypted_title = crypto.decrypt(&row.0, &salt).unwrap();
    assert_eq!(decrypted_title, "Test Note Title");

    // Delete
    let del_result = sqlx
        ::query("DELETE FROM notes WHERE id = ?")
        .bind(&note_id)
        .execute(&pool).await
        .expect("DELETE note");
    assert_eq!(del_result.rows_affected(), 1);

    // Cleanup user
    sqlx::query("DELETE FROM users WHERE id = ?").bind(&user_id).execute(&pool).await.ok();
}

// ═══════════════════════════════════════════════════════════
//  Chat CRUD Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_chat_lifecycle() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let chat_id = uuid::Uuid::new_v4().to_string();
    let salt = bsdy_api::crypto::CryptoService::generate_user_salt();

    // Insert user
    sqlx::query(
        r#"INSERT INTO users (id, google_id, username, name, email, email_verification_status,
           onboarding_completed, encryption_salt)
           VALUES (?, ?, ?, ?, ?, 'verified', TRUE, ?)"#
    )
        .bind(&user_id)
        .bind("google-chat-1")
        .bind(&format!("chatuser_{}", &user_id[..6]))
        .bind("Chat User")
        .bind(&format!("chat-{}@test.com", &user_id[..8]))
        .bind(&salt)
        .execute(&pool).await
        .unwrap();

    // Create chat
    sqlx::query(
        "INSERT INTO chats (id, user_id, title, chat_type, is_active, message_count) VALUES (?, ?, 'Test Chat', 'companion', TRUE, 0)"
    )
        .bind(&chat_id)
        .bind(&user_id)
        .execute(&pool).await
        .expect("INSERT chat");

    // Verify
    let row: (String, String, bool) = sqlx
        ::query_as("SELECT id, chat_type, is_active FROM chats WHERE id = ?")
        .bind(&chat_id)
        .fetch_one(&pool).await
        .expect("SELECT chat");

    assert_eq!(row.0, chat_id);
    assert_eq!(row.1, "companion");
    assert!(row.2);

    // Delete cascade (messages → chat)
    sqlx::query("DELETE FROM chats WHERE id = ?").bind(&chat_id).execute(&pool).await.ok();
    sqlx::query("DELETE FROM users WHERE id = ?").bind(&user_id).execute(&pool).await.ok();
}

// ═══════════════════════════════════════════════════════════
//  Scheduled Tasks Test
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_db_scheduled_task_logging() {
    let pool = test_pool().await;
    db::run_migrations(&pool).await.unwrap();

    let task_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO scheduled_tasks (id, task_type, next_run_at, status, details)
           VALUES (?, 'weekly_report', NOW(), 'completed', '{"total_users":5,"success":5,"errors":0}')"#
    )
        .bind(&task_id)
        .execute(&pool).await
        .expect("INSERT scheduled_task");

    let row: (String, String) = sqlx
        ::query_as("SELECT task_type, status FROM scheduled_tasks WHERE id = ?")
        .bind(&task_id)
        .fetch_one(&pool).await
        .expect("SELECT scheduled_task");

    assert_eq!(row.0, "weekly_report");
    assert_eq!(row.1, "completed");

    // Cleanup
    sqlx::query("DELETE FROM scheduled_tasks WHERE id = ?")
        .bind(&task_id)
        .execute(&pool).await
        .ok();
}
