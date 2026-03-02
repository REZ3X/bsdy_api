//! Tests for GeminiService using Wiremock to simulate the Gemini API.

mod common;

use bsdy_api::services::gemini_service::GeminiService;
use common::*;

// ═══════════════════════════════════════════════════════════
//  GeminiService Construction Tests
// ═══════════════════════════════════════════════════════════

#[test]
fn test_gemini_service_creation() {
    let service = GeminiService::new("api-key-123".into(), "gemini-3.1-pro-preview".into());
    // Service should be cloneable (for Arc usage)
    let _cloned = service.clone();
}

#[test]
fn test_gemini_service_clone_is_independent() {
    let service1 = GeminiService::new("api-key-1".into(), "model-1".into());
    let service2 = service1.clone();
    // Both should work independently (they share the same internal Arc<Client>)
    let _s2 = service2; // no panic
}

// ═══════════════════════════════════════════════════════════
//  GeminiService Live Integration Tests (ignored by default)
// ═══════════════════════════════════════════════════════════

/// This test actually calls the Gemini API — requires GEMINI_API_KEY env var.
/// Run with: cargo test test_gemini_live -- --ignored
#[tokio::test]
#[ignore]
async fn test_gemini_live_generate_text() {
    load_env();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY required for live test");
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-pro-preview".into());
    let service = GeminiService::new(api_key, model);

    let result = service.generate_text("Say hello in exactly 3 words.").await;
    assert!(result.is_ok(), "Gemini API call failed: {:?}", result.err());
    let text = result.unwrap();
    assert!(!text.is_empty(), "Gemini response should not be empty");
    println!("Gemini response: {}", text);
}

/// Test system instruction support.
#[tokio::test]
#[ignore]
async fn test_gemini_live_generate_with_system() {
    load_env();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY required for live test");
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-pro-preview".into());
    let service = GeminiService::new(api_key, model);

    let result = service.generate_with_system(
        "Tell me a fun fact",
        Some("You are a playful science teacher. Always start with 'Fun fact!'"),
        0.7,
        256
    ).await;
    assert!(result.is_ok(), "System instruction call failed: {:?}", result.err());
    let text = result.unwrap();
    println!("Gemini with system: {}", text);
}

/// Test multi-turn conversation.
#[tokio::test]
#[ignore]
async fn test_gemini_live_chat_response() {
    load_env();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY required for live test");
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-pro-preview".into());
    let service = GeminiService::new(api_key, model);

    let history = vec![
        ("user".to_string(), "My name is Alex".to_string()),
        ("model".to_string(), "Nice to meet you, Alex! How are you feeling today?".to_string())
    ];

    let result = service.generate_chat_response(
        "You are a compassionate mental health companion.",
        &history,
        "I've been feeling stressed about work",
        0.8
    ).await;
    assert!(result.is_ok(), "Chat response failed: {:?}", result.err());
    let text = result.unwrap();
    assert!(!text.is_empty());
    println!("Gemini chat: {}", text);
}

/// Test chat title generation.
#[tokio::test]
#[ignore]
async fn test_gemini_live_generate_chat_title() {
    load_env();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY required for live test");
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-pro-preview".into());
    let service = GeminiService::new(api_key, model);

    let result = service.generate_chat_title(
        "I've been having trouble sleeping and feeling anxious about my exams"
    ).await;
    assert!(result.is_ok(), "Title generation failed: {:?}", result.err());
    let title = result.unwrap();
    assert!(!title.is_empty());
    assert!(title.len() <= 60, "Title should be max 60 chars");
    println!("Generated title: {}", title);
}

/// Test mood data analysis.
#[tokio::test]
#[ignore]
async fn test_gemini_live_analyze_mood_data() {
    load_env();
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY required for live test");
    let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.1-pro-preview".into());
    let service = GeminiService::new(api_key, model);

    let mood_json =
        r#"[
        {"mood_score": 6, "energy_level": 5, "anxiety_level": 3, "stress_level": 4},
        {"mood_score": 4, "energy_level": 3, "anxiety_level": 6, "stress_level": 7},
        {"mood_score": 7, "energy_level": 6, "anxiety_level": 2, "stress_level": 3}
    ]"#;
    let baseline_json = r#"{"risk_level":"moderate","baseline_stress":"moderate"}"#;

    let result = service.analyze_mood_data("TestUser", mood_json, baseline_json, "weekly").await;
    assert!(result.is_ok(), "Mood analysis failed: {:?}", result.err());
    let text = result.unwrap();
    assert!(!text.is_empty());
    println!("Mood analysis: {}", &text[..text.len().min(200)]);
}

// ═══════════════════════════════════════════════════════════
//  GeminiService Error Handling Tests
// ═══════════════════════════════════════════════════════════

#[tokio::test]
async fn test_gemini_invalid_key_returns_error() {
    // This test actually tries to call Google with a bad key to confirm error handling
    let service = GeminiService::new("invalid-key-12345".into(), "gemini-3.1-pro-preview".into());
    let result = service.generate_text("Hello").await;
    assert!(result.is_err(), "Should fail with invalid API key");
}
