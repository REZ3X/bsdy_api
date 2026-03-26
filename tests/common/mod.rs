//! Shared test helpers, mock factories, and test configuration.
#![allow(dead_code)]

use bsdy_api::config::*;
use bsdy_api::crypto::CryptoService;
use bsdy_api::services::email_service::EmailService;
use bsdy_api::services::gemini_service::GeminiService;

pub fn load_env() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let env_path = std::path::Path::new(manifest_dir).join(".env");
    dotenvy::from_path(&env_path).ok();
}

/// A valid 64-hex-char master key for testing (32 bytes).
pub const TEST_MASTER_KEY: &str =
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

/// Build a test Config with all required fields set to sensible test defaults.
pub fn test_config() -> Config {
    Config {
        app: AppConfig {
            name: "BSDY-Test".into(),
            env: "test".into(),
            port: 0, // bind to random port
            mode: "internal".into(),
            frontend_url: "http://localhost:3000".into(),
        },
        database: DatabaseConfig {
            url: test_database_url(),
            max_connections: 2,
        },
        jwt: JwtConfig {
            secret: "test-jwt-secret-that-is-long-enough-for-hmac".into(),
            expiration_hours: 72,
        },
        google_oauth: GoogleOAuthConfig {
            client_id: "test-google-client-id".into(),
            client_secret: "test-google-client-secret".into(),
            redirect_uri: "http://localhost:8000/api/auth/google/callback".into(),
        },
        brevo: BrevoConfig {
            smtp_host: "localhost".into(),
            smtp_port: 2525,
            smtp_user: "test".into(),
            smtp_pass: "test".into(),
            from_email: "test@bsdy.app".into(),
            from_name: "BSDY Test".into(),
        },
        gemini: GeminiConfig {
            api_key: "test-gemini-key".into(),
            model: "gemini-3-flash-preview".into(),
        },
        encryption: EncryptionConfig {
            master_key: TEST_MASTER_KEY.into(),
        },
        security: SecurityConfig {
            api_key: "test-api-key-12345".into(),
        },
        scheduler: SchedulerConfig {
            weekly_report_cron: "0 0 9 * * Mon".into(),
            monthly_report_cron: "0 0 9 1 * *".into(),
            yearly_report_cron: "0 0 9 1 1 *".into(),
        },
        docs: DocsConfig {
            password: "test-docs-pass".into(),
        },
    }
}

pub fn test_database_url() -> String {
    std::env
        ::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| {
            std::env
                ::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:@localhost:3306/bsdy_test".into())
        })
}

pub fn test_crypto() -> CryptoService {
    CryptoService::new(TEST_MASTER_KEY).expect("test crypto init")
}

pub fn test_gemini(_base_url: &str) -> GeminiService {
            GeminiService::new("fake-key".into(), "gemini-test".into())
}

pub fn test_email() -> EmailService {
    let brevo = BrevoConfig {
        smtp_host: "localhost".into(),
        smtp_port: 2525,
        smtp_user: "test".into(),
        smtp_pass: "test".into(),
        from_email: "test@bsdy.app".into(),
        from_name: "BSDY Test".into(),
    };
    EmailService::new(&brevo, "BSDY-Test", "http://localhost:3000")
}

/// Generate a random user salt for testing.
pub fn random_salt() -> String {
    CryptoService::generate_user_salt()
}
