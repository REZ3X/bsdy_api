//! Tests for AuthService — JWT generation, verification tokens, Google OAuth URL,
//! and the ChatService severity detection logic.

mod common;

use bsdy_api::models::user::{ Claims, UserRow };
use bsdy_api::services::auth_service::AuthService;
use bsdy_api::services::chat_service::ChatService;
use chrono::NaiveDateTime;
use common::*;
use jsonwebtoken::{ decode, DecodingKey, Validation };

// ═══════════════════════════════════════════════════════════
//  JWT Tests
// ═══════════════════════════════════════════════════════════

fn make_test_user() -> UserRow {
    let now = NaiveDateTime::parse_from_str("2026-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    UserRow {
        id: "user-test-123".into(),
        google_id: "google-456".into(),
        username: "testuser".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        avatar_url: None,
        birth: None,
        email_verification_status: "verified".into(),
        email_verification_token: None,
        email_verified_at: Some(now),
        onboarding_completed: true,
        encryption_salt: "abcdef1234567890".into(),
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn test_generate_jwt_success() {
    let config = test_config();
    let user = make_test_user();
    let token = AuthService::generate_jwt(&user, &config);
    assert!(token.is_ok(), "JWT generation should succeed");
    let token = token.unwrap();
    assert!(!token.is_empty());
    // Token should have 3 parts separated by dots
    assert_eq!(token.split('.').count(), 3, "JWT should have 3 parts");
}

#[test]
fn test_generated_jwt_contains_correct_claims() {
    let config = test_config();
    let user = make_test_user();
    let token = AuthService::generate_jwt(&user, &config).unwrap();

    let mut validation = Validation::default();
    validation.validate_exp = false; // Don't fail on expiry for test
    validation.required_spec_claims.clear();

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation
    ).expect("should decode generated JWT");

    assert_eq!(decoded.claims.sub, "user-test-123");
    assert_eq!(decoded.claims.email, "test@example.com");
    assert!(decoded.claims.exp > decoded.claims.iat);
}

#[test]
fn test_jwt_expiration_is_correct() {
    let config = test_config();
    let user = make_test_user();
    let token = AuthService::generate_jwt(&user, &config).unwrap();

    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation
    ).unwrap();

    let expected_duration = config.jwt.expiration_hours * 3600;
    let actual_duration = decoded.claims.exp - decoded.claims.iat;
    assert_eq!(actual_duration, expected_duration);
}

#[test]
fn test_jwt_invalid_secret_fails_decode() {
    let config = test_config();
    let user = make_test_user();
    let token = AuthService::generate_jwt(&user, &config).unwrap();

    let mut validation = Validation::default();
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let result = decode::<Claims>(&token, &DecodingKey::from_secret(b"wrong-secret"), &validation);
    assert!(result.is_err(), "JWT should fail with wrong secret");
}

// ═══════════════════════════════════════════════════════════
//  Google OAuth URL Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_google_auth_url_generation() {
    let config = test_config();
    let url = AuthService::google_auth_url(&config);
    assert!(url.is_ok());
    let url = url.unwrap();
    assert!(url.contains("accounts.google.com"), "should contain Google domain");
    assert!(url.contains("response_type"), "should contain response_type param");
    assert!(url.contains(&config.google_oauth.client_id), "should contain client_id");
}

// ═══════════════════════════════════════════════════════════
//  Verification Token Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_verification_token_length() {
    let token = AuthService::generate_verification_token();
    assert_eq!(token.len(), 48, "token should be 48 chars");
}

#[test]
fn test_verification_token_is_alphanumeric() {
    let token = AuthService::generate_verification_token();
    assert!(
        token.chars().all(|c| c.is_alphanumeric()),
        "token should be alphanumeric"
    );
}

#[test]
fn test_verification_tokens_are_unique() {
    let t1 = AuthService::generate_verification_token();
    let t2 = AuthService::generate_verification_token();
    assert_ne!(t1, t2, "tokens should be unique");
}

// ═══════════════════════════════════════════════════════════
//  Chat Severity Detection Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_detect_severity_crisis_keywords() {
    assert_eq!(ChatService::detect_severity("I want to kill myself"), "crisis");
    assert_eq!(ChatService::detect_severity("I want to end my life"), "crisis");
    assert_eq!(ChatService::detect_severity("thinking about suicide"), "crisis");
    assert_eq!(ChatService::detect_severity("I want to die"), "crisis");
    assert_eq!(ChatService::detect_severity("I've been self-harm"), "crisis");
}

#[test]
fn test_detect_severity_severe_keywords() {
    assert_eq!(ChatService::detect_severity("I feel hopeless"), "severe");
    assert_eq!(ChatService::detect_severity("I feel worthless"), "severe");
    assert_eq!(ChatService::detect_severity("I'm giving up on everything"), "severe");
    assert_eq!(ChatService::detect_severity("nothing matters anymore"), "severe");
}

#[test]
fn test_detect_severity_none() {
    assert_eq!(ChatService::detect_severity("I had a good day today"), "none");
    assert_eq!(ChatService::detect_severity("feeling a bit anxious"), "none");
    assert_eq!(ChatService::detect_severity("how's the weather?"), "none");
}

#[test]
fn test_detect_severity_case_insensitive() {
    assert_eq!(ChatService::detect_severity("I WANT TO DIE"), "crisis");
    assert_eq!(ChatService::detect_severity("HOPELESS"), "severe");
}

#[test]
fn test_detect_severity_crisis_takes_priority() {
    // Message contains both crisis and severe keywords
    assert_eq!(ChatService::detect_severity("I feel hopeless and want to kill myself"), "crisis");
}
