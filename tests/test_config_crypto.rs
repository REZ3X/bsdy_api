//! Tests for Config loading and CryptoService (encryption/decryption).

mod common;

use bsdy_api::crypto::CryptoService;
use common::*;

// ═══════════════════════════════════════════════════════════
//  Config Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_config_test_defaults_are_valid() {
    let cfg = test_config();
    assert_eq!(cfg.app.name, "BSDY-Test");
    assert_eq!(cfg.app.env, "test");
    assert_eq!(cfg.app.mode, "internal");
    assert!(!cfg.is_external());
    assert!(!cfg.is_production());
}

#[test]
fn test_config_is_external_mode() {
    let mut cfg = test_config();
    cfg.app.mode = "external".into();
    assert!(cfg.is_external());
}

#[test]
fn test_config_is_production() {
    let mut cfg = test_config();
    cfg.app.env = "production".into();
    assert!(cfg.is_production());
}

#[test]
fn test_config_scheduler_crons_present() {
    let cfg = test_config();
    assert!(!cfg.scheduler.weekly_report_cron.is_empty());
    assert!(!cfg.scheduler.monthly_report_cron.is_empty());
    assert!(!cfg.scheduler.yearly_report_cron.is_empty());
}

// ═══════════════════════════════════════════════════════════
//  CryptoService Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_crypto_new_valid_key() {
    let crypto = CryptoService::new(TEST_MASTER_KEY);
    assert!(crypto.is_ok());
}

#[test]
fn test_crypto_new_invalid_hex() {
    let result = CryptoService::new("not-valid-hex");
    assert!(result.is_err());
}

#[test]
fn test_crypto_new_wrong_key_length() {
    // Only 16 bytes (32 hex chars) instead of required 32 bytes (64 hex chars)
    let result = CryptoService::new("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4");
    assert!(result.is_err());
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let crypto = test_crypto();
    let salt = random_salt();
    let plaintext = "Hello, this is sensitive mental health data!";

    let encrypted = crypto.encrypt(plaintext, &salt).unwrap();
    assert_ne!(encrypted, plaintext, "encrypted text should differ from plaintext");

    let decrypted = crypto.decrypt(&encrypted, &salt).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_produces_different_ciphertext_each_time() {
    let crypto = test_crypto();
    let salt = random_salt();
    let plaintext = "Same data encrypted twice";

    let enc1 = crypto.encrypt(plaintext, &salt).unwrap();
    let enc2 = crypto.encrypt(plaintext, &salt).unwrap();
    // Due to random nonce, encryptions should differ
    assert_ne!(enc1, enc2, "two encryptions of same data should differ (random nonce)");

    // But both should decrypt to the same plaintext
    assert_eq!(crypto.decrypt(&enc1, &salt).unwrap(), plaintext);
    assert_eq!(crypto.decrypt(&enc2, &salt).unwrap(), plaintext);
}

#[test]
fn test_different_salts_cannot_cross_decrypt() {
    let crypto = test_crypto();
    let salt1 = "user-salt-aaaa";
    let salt2 = "user-salt-bbbb";
    let plaintext = "Secret data";

    let encrypted = crypto.encrypt(plaintext, salt1).unwrap();
    // Decrypting with wrong salt should fail
    let result = crypto.decrypt(&encrypted, salt2);
    assert!(result.is_err(), "decrypting with wrong salt should fail");
}

#[test]
fn test_encrypt_optional_none() {
    let crypto = test_crypto();
    let salt = random_salt();
    let result = crypto.encrypt_optional(None, &salt).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_encrypt_optional_some() {
    let crypto = test_crypto();
    let salt = random_salt();
    let plaintext = "Optional field";

    let encrypted = crypto.encrypt_optional(Some(plaintext), &salt).unwrap();
    assert!(encrypted.is_some());

    let decrypted = crypto.decrypt_optional(encrypted.as_deref(), &salt).unwrap();
    assert_eq!(decrypted.as_deref(), Some(plaintext));
}

#[test]
fn test_decrypt_optional_none() {
    let crypto = test_crypto();
    let salt = random_salt();
    let result = crypto.decrypt_optional(None, &salt).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_generate_user_salt_uniqueness() {
    let salt1 = CryptoService::generate_user_salt();
    let salt2 = CryptoService::generate_user_salt();
    assert_ne!(salt1, salt2, "generated salts should be unique");
    assert_eq!(salt1.len(), 32, "salt should be 32 hex chars (16 bytes)");
}

#[test]
fn test_decrypt_invalid_base64() {
    let crypto = test_crypto();
    let salt = random_salt();
    let result = crypto.decrypt("not-valid-base64!!!", &salt);
    assert!(result.is_err());
}

#[test]
fn test_decrypt_too_short_ciphertext() {
    let crypto = test_crypto();
    let salt = random_salt();
    // Valid base64 but only a few bytes (less than 12-byte nonce)
    let short = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[0u8; 5]);
    let result = crypto.decrypt(&short, &salt);
    assert!(result.is_err());
}

#[test]
fn test_encrypt_empty_string() {
    let crypto = test_crypto();
    let salt = random_salt();
    let encrypted = crypto.encrypt("", &salt).unwrap();
    let decrypted = crypto.decrypt(&encrypted, &salt).unwrap();
    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_unicode() {
    let crypto = test_crypto();
    let salt = random_salt();
    let plaintext = "こんにちは世界 أهلاً بالعالم";
    let encrypted = crypto.encrypt(plaintext, &salt).unwrap();
    let decrypted = crypto.decrypt(&encrypted, &salt).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_large_payload() {
    let crypto = test_crypto();
    let salt = random_salt();
    // 100KB of data
    let plaintext: String = "A".repeat(100_000);
    let encrypted = crypto.encrypt(&plaintext, &salt).unwrap();
    let decrypted = crypto.decrypt(&encrypted, &salt).unwrap();
    assert_eq!(decrypted, plaintext);
}
