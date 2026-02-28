use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub google_oauth: GoogleOAuthConfig,
    pub brevo: BrevoConfig,
    pub gemini: GeminiConfig,
    pub encryption: EncryptionConfig,
    pub security: SecurityConfig,
    pub scheduler: SchedulerConfig,
    pub docs: DocsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub env: String,
    pub port: u16,
    pub mode: String,
    pub frontend_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrevoConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_email: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EncryptionConfig {
    pub master_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    pub weekly_report_cron: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocsConfig {
    pub password: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Config {
            app: AppConfig {
                name: std::env::var("APP_NAME").unwrap_or_else(|_| "BSDY".into()),
                env: std::env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
                port: std::env
                    ::var("APP_PORT")
                    .unwrap_or_else(|_| "8000".into())
                    .parse()?,
                mode: std::env::var("APP_MODE").unwrap_or_else(|_| "internal".into()),
                frontend_url: std::env
                    ::var("FRONTEND_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".into()),
            },
            database: DatabaseConfig {
                url: std::env
                    ::var("DATABASE_URL")
                    .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
                max_connections: std::env
                    ::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".into())
                    .parse()?,
            },
            jwt: JwtConfig {
                secret: std::env
                    ::var("JWT_SECRET")
                    .map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?,
                expiration_hours: std::env
                    ::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "72".into())
                    .parse()?,
            },
            google_oauth: GoogleOAuthConfig {
                client_id: std::env
                    ::var("GOOGLE_CLIENT_ID")
                    .map_err(|_| anyhow::anyhow!("GOOGLE_CLIENT_ID is required"))?,
                client_secret: std::env
                    ::var("GOOGLE_CLIENT_SECRET")
                    .map_err(|_| anyhow::anyhow!("GOOGLE_CLIENT_SECRET is required"))?,
                redirect_uri: std::env
                    ::var("GOOGLE_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:8000/api/auth/google/callback".into()),
            },
            brevo: BrevoConfig {
                smtp_host: std::env
                    ::var("BREVO_SMTP_HOST")
                    .unwrap_or_else(|_| "smtp-relay.brevo.com".into()),
                smtp_port: std::env
                    ::var("BREVO_SMTP_PORT")
                    .unwrap_or_else(|_| "587".into())
                    .parse()?,
                smtp_user: std::env
                    ::var("BREVO_SMTP_USER")
                    .map_err(|_| anyhow::anyhow!("BREVO_SMTP_USER is required"))?,
                smtp_pass: std::env
                    ::var("BREVO_SMTP_PASS")
                    .map_err(|_| anyhow::anyhow!("BREVO_SMTP_PASS is required"))?,
                from_email: std::env
                    ::var("BREVO_FROM_EMAIL")
                    .unwrap_or_else(|_| "noreply@bsdy.app".into()),
                from_name: std::env
                    ::var("BREVO_FROM_NAME")
                    .unwrap_or_else(|_| "BSDY Mental Companion".into()),
            },
            gemini: GeminiConfig {
                api_key: std::env
                    ::var("GEMINI_API_KEY")
                    .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY is required"))?,
                model: std::env
                    ::var("GEMINI_MODEL")
                    .unwrap_or_else(|_| "gemini-2.5-pro-preview-05-06".into()),
            },
            encryption: EncryptionConfig {
                master_key: std::env
                    ::var("ENCRYPTION_MASTER_KEY")
                    .map_err(|_| anyhow::anyhow!("ENCRYPTION_MASTER_KEY is required"))?,
            },
            security: SecurityConfig {
                api_key: std::env::var("API_KEY").unwrap_or_else(|_| "".into()),
            },
            scheduler: SchedulerConfig {
                weekly_report_cron: std::env
                    ::var("WEEKLY_REPORT_CRON")
                    .unwrap_or_else(|_| "0 0 9 * * Mon".into()),
            },
            docs: DocsConfig {
                password: std::env
                    ::var("DOCS_PASSWORD")
                    .unwrap_or_else(|_| "bsdy-docs-2026".into()),
            },
        })
    }

    pub fn is_external(&self) -> bool {
        self.app.mode == "external"
    }

    pub fn is_production(&self) -> bool {
        self.app.env == "production"
    }
}
