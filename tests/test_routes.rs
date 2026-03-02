//! Integration tests for HTTP routes and the scheduler configuration.
//!
//! Health endpoint tests run without a real database (they use the router directly).
//! Full integration tests require a running MariaDB — run with `--ignored`.

mod common;

use axum::body::Body;
use axum::http::{ Request, StatusCode };
use bsdy_api::routes::build_router;
use bsdy_api::state::AppState;
use bsdy_api::services::gemini_service::GeminiService;
use common::*;
use tower::ServiceExt; // for `oneshot`

// ═══════════════════════════════════════════════════════════
//  Helper: build an in-memory router for route testing
// ═══════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════
//  Health Endpoint Tests (with DB)
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_health_endpoint() {
    load_env();
    let url = test_database_url();
    let config_db = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = bsdy_api::db::create_pool(&config_db).await.expect("pool");
    let config = test_config();
    let crypto = test_crypto();
    let gemini = GeminiService::new("fake".into(), "fake".into());
    let email = test_email();
    let state = AppState::new(pool, config, crypto, gemini, email);

    let app = build_router().with_state(state);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ═══════════════════════════════════════════════════════════
//  Auth Routes Structure Tests (with DB)
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_google_auth_redirect_returns_url() {
    load_env();
    let url = test_database_url();
    let config_db = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = bsdy_api::db::create_pool(&config_db).await.expect("pool");
    let config = test_config();
    let crypto = test_crypto();
    let gemini = GeminiService::new("fake".into(), "fake".into());
    let email = test_email();
    let state = AppState::new(pool, config, crypto, gemini, email);

    let app = build_router().with_state(state);

    let response = app
        .oneshot(Request::builder().uri("/api/auth/google/url").body(Body::empty()).unwrap()).await
        .unwrap();

    // Should return 200 with auth_url (or redirect)
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::TEMPORARY_REDIRECT,
        "Google auth should return 200 or 307, got {}",
        response.status()
    );
}

// ═══════════════════════════════════════════════════════════
//  Protected Routes — Should Require Auth
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_protected_routes_require_auth() {
    load_env();
    let url = test_database_url();
    let config_db = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = bsdy_api::db::create_pool(&config_db).await.expect("pool");
    let config = test_config();
    let crypto = test_crypto();
    let gemini = GeminiService::new("fake".into(), "fake".into());
    let email = test_email();
    let state = AppState::new(pool, config, crypto, gemini, email);

    let protected_routes = vec![
        "/api/auth/me",
        "/api/onboarding/baseline",
        "/api/mood",
        "/api/mood/today",
        "/api/analytics",
        "/api/reports",
        "/api/notes",
        "/api/chats",
        "/api/logs/auth",
        "/api/logs/activity"
    ];

    for route in &protected_routes {
        let app = build_router().with_state(state.clone());
        let response = app
            .oneshot(Request::builder().uri(*route).body(Body::empty()).unwrap()).await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "Route {} should return 401 without auth token, got {}",
            route,
            response.status()
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  API Key Middleware Tests (External Mode)
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_external_mode_requires_api_key() {
    load_env();
    let url = test_database_url();
    let config_db = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = bsdy_api::db::create_pool(&config_db).await.expect("pool");
    let mut config = test_config();
    config.app.mode = "external".into(); // enable API key check
    let crypto = test_crypto();
    let gemini = GeminiService::new("fake".into(), "fake".into());
    let email = test_email();
    let state = AppState::new(pool, config, crypto, gemini, email);

    let app = build_router()
        .layer(
            axum::middleware::from_fn_with_state(
                state.clone(),
                bsdy_api::middleware::api_key::api_key_layer
            )
        )
        .with_state(state);

    // Without API key header — hit a route NOT exempt from API key check
    // (/health, /api/auth/*, /docs are exempt in api_key_layer)
    let response = app
        .oneshot(Request::builder().uri("/api/mood").body(Body::empty()).unwrap()).await
        .unwrap();

    // Should be 401 from the API key middleware
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN,
        "External mode without API key should reject, got {}",
        response.status()
    );
}

// ═══════════════════════════════════════════════════════════
//  Docs Endpoint Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
#[ignore]
async fn test_docs_endpoint_returns_html() {
    load_env();
    let url = test_database_url();
    let config_db = bsdy_api::config::DatabaseConfig {
        url,
        max_connections: 2,
    };
    let pool = bsdy_api::db::create_pool(&config_db).await.expect("pool");
    let config = test_config();
    let crypto = test_crypto();
    let gemini = GeminiService::new("fake".into(), "fake".into());
    let email = test_email();
    let state = AppState::new(pool, config, crypto, gemini, email);

    let app = build_router().with_state(state);

    let response = app
        .oneshot(
            Request::builder().uri("/docs?password=test-docs-pass").body(Body::empty()).unwrap()
        ).await
        .unwrap();

    // Docs should return 200 with correct password
    assert_eq!(response.status(), StatusCode::OK);
}

// ═══════════════════════════════════════════════════════════
//  Scheduler Config Tests (no DB needed)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_scheduler_config_cron_expressions_are_valid() {
    let config = test_config();

    // Basic validation: cron expressions should have 6 fields (sec min hour dom mon dow)
    for (label, cron) in [
        ("weekly", &config.scheduler.weekly_report_cron),
        ("monthly", &config.scheduler.monthly_report_cron),
        ("yearly", &config.scheduler.yearly_report_cron),
    ] {
        let parts: Vec<&str> = cron.split_whitespace().collect();
        assert_eq!(
            parts.len(),
            6,
            "{} cron '{}' should have 6 fields, got {}",
            label,
            cron,
            parts.len()
        );
    }
}

#[test]
fn test_scheduler_weekly_cron_runs_on_monday() {
    let config = test_config();
    assert!(
        config.scheduler.weekly_report_cron.contains("Mon"),
        "weekly cron should reference Monday"
    );
}

#[test]
fn test_scheduler_monthly_cron_runs_on_first() {
    let config = test_config();
    // "0 0 9 1 * *" — 4th field is day-of-month = 1
    let parts: Vec<&str> = config.scheduler.monthly_report_cron.split_whitespace().collect();
    assert_eq!(parts[3], "1", "monthly cron should run on the 1st");
}

#[test]
fn test_scheduler_yearly_cron_runs_jan_first() {
    let config = test_config();
    // "0 0 9 1 1 *" — 4th field = 1 (day), 5th field = 1 (Jan)
    let parts: Vec<&str> = config.scheduler.yearly_report_cron.split_whitespace().collect();
    assert_eq!(parts[3], "1", "yearly cron should run on the 1st");
    assert_eq!(parts[4], "1", "yearly cron should run in January");
}
