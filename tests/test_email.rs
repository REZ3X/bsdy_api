//! Tests for EmailService — construction, message building, and live SMTP tests.

mod common;

use bsdy_api::config::BrevoConfig;
use bsdy_api::services::email_service::EmailService;
use common::*;

// ═══════════════════════════════════════════════════════════
//  EmailService Construction Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_email_service_creation() {
    let service = test_email();
    // Should construct without panicking
    let _cloned = service.clone();
}

#[test]
fn test_email_service_from_brevo_config() {
    let brevo = BrevoConfig {
        smtp_host: "smtp-relay.brevo.com".into(),
        smtp_port: 587,
        smtp_user: "user@test.com".into(),
        smtp_pass: "secret-pass".into(),
        from_email: "noreply@bsdy.app".into(),
        from_name: "BSDY Mental Companion".into(),
    };
    let _service = EmailService::new(&brevo, "BSDY", "https://bsdy.app");
}

// ═══════════════════════════════════════════════════════════
//  EmailService Live Tests (ignored by default)
//  These actually send email through Brevo SMTP.
//  Run with: cargo test test_email_live -- --ignored
// ═══════════════════════════════════════════════════════════

/// Helper to create a live email service from env vars.
fn live_email_service() -> EmailService {
    load_env();
    let brevo = BrevoConfig {
        smtp_host: std::env
            ::var("BREVO_SMTP_HOST")
            .unwrap_or_else(|_| "smtp-relay.brevo.com".into()),
        smtp_port: std::env
            ::var("BREVO_SMTP_PORT")
            .unwrap_or_else(|_| "587".into())
            .parse()
            .unwrap(),
        smtp_user: std::env::var("BREVO_SMTP_USER").expect("BREVO_SMTP_USER required"),
        smtp_pass: std::env::var("BREVO_SMTP_PASS").expect("BREVO_SMTP_PASS required"),
        from_email: std::env::var("BREVO_FROM_EMAIL").unwrap_or_else(|_| "noreply@bsdy.app".into()),
        from_name: std::env::var("BREVO_FROM_NAME").unwrap_or_else(|_| "BSDY Test".into()),
    };
    EmailService::new(
        &brevo,
        "BSDY-Test",
        &std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".into())
    )
}

#[tokio::test]
#[ignore]
async fn test_email_live_send_verification() {
    let service = live_email_service();
    let test_email = std::env
        ::var("TEST_EMAIL_RECIPIENT")
        .expect("TEST_EMAIL_RECIPIENT required for live email test");

    let result = service.send_verification_email(
        &test_email,
        "Test User",
        "fake-token-12345"
    ).await;

    assert!(result.is_ok(), "Verification email send failed: {:?}", result.err());
    println!("[OK] Verification email sent to {}", test_email);
}

#[tokio::test]
#[ignore]
async fn test_email_live_send_report() {
    let service = live_email_service();
    let test_email = std::env
        ::var("TEST_EMAIL_RECIPIENT")
        .expect("TEST_EMAIL_RECIPIENT required for live email test");

    let result = service.send_report_email(
        &test_email,
        "Test User",
        "weekly",
        "2026-02-23",
        "2026-03-01",
        "Your mood has been generally stable this week with minor fluctuations.",
        "1. Continue your daily journaling\n2. Try 10 minutes of meditation\n3. Maintain your sleep schedule",
        "stable",
        Some(7.2),
        "low"
    ).await;

    assert!(result.is_ok(), "Report email send failed: {:?}", result.err());
    println!("[OK] Report email sent to {}", test_email);
}

#[tokio::test]
#[ignore]
async fn test_email_live_send_crisis_alert() {
    let service = live_email_service();
    let test_email = std::env
        ::var("TEST_EMAIL_RECIPIENT")
        .expect("TEST_EMAIL_RECIPIENT required for live email test");

    let result = service.send_crisis_alert(&test_email, "Test User").await;

    assert!(result.is_ok(), "Crisis alert email send failed: {:?}", result.err());
    println!("[OK] Crisis alert email sent to {}", test_email);
}
