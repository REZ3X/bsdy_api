use std::sync::Arc;

use sqlx::MySqlPool;

use crate::{
    config::Config,
    crypto::CryptoService,
    services::{ email_service::EmailService, gemini_service::GeminiService },
};

/// Shared application state available in all route handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub config: Arc<Config>,
    pub crypto: Arc<CryptoService>,
    pub gemini: Arc<GeminiService>,
    pub email: Arc<EmailService>,
    pub http_client: Arc<reqwest::Client>,
}

impl AppState {
    pub fn new(
        db: MySqlPool,
        config: Config,
        crypto: CryptoService,
        gemini: GeminiService,
        email: EmailService
    ) -> Self {
        Self {
            db,
            config: Arc::new(config),
            crypto: Arc::new(crypto),
            gemini: Arc::new(gemini),
            email: Arc::new(email),
            http_client: Arc::new(
                reqwest::Client
                    ::builder()
                    .timeout(std::time::Duration::from_secs(60))
                    .pool_max_idle_per_host(10)
                    .build()
                    .expect("Failed to create HTTP client")
            ),
        }
    }
}
