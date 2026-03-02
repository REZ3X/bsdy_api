use std::net::SocketAddr;

use axum::middleware as axum_mw;
use tower_http::{ cors::{ Any, CorsLayer }, trace::TraceLayer };
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt, EnvFilter };

use bsdy_api::{
    config::Config,
    crypto::CryptoService,
    db,
    middleware::api_key::api_key_layer,
    routes::build_router,
    services::{ EmailService, GeminiService, SchedulerService },
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── 1. Tracing / Logging ────────────────────────────────
    tracing_subscriber
        ::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_|
                "bsdy_api=debug,tower_http=debug".into()
            )
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting BSDY API server...");

    // ── 2. Configuration ────────────────────────────────────
    let config = Config::from_env()?;
    tracing::info!(
        "Config loaded — env: {}, mode: {}, port: {}",
        config.app.env,
        config.app.mode,
        config.app.port
    );

    // ── 3. Database ─────────────────────────────────────────
    let pool = db::create_pool(&config.database).await?;
    db::run_migrations(&pool).await?;

    // ── 4. Services ─────────────────────────────────────────
    let crypto = CryptoService::new(&config.encryption.master_key)?;
    let gemini = GeminiService::new(config.gemini.api_key.clone(), config.gemini.model.clone());
    let email = EmailService::new(&config.brevo, &config.app.name, &config.app.frontend_url);

    // ── 5. App State ────────────────────────────────────────
    let state = AppState::new(pool.clone(), config.clone(), crypto, gemini, email);

    // ── 6. Background Scheduler ─────────────────────────────
    let _scheduler = SchedulerService::start(
        state.db.clone(),
        state.config.clone(),
        state.crypto.clone(),
        state.gemini.clone(),
        state.email.clone()
    ).await?;
    tracing::info!("Background scheduler started");

    // ── 7. CORS ─────────────────────────────────────────────
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // ── 8. Build Router ─────────────────────────────────────
    let app = build_router()
        .layer(axum_mw::from_fn_with_state(state.clone(), api_key_layer))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // ── 9. Start Server ─────────────────────────────────────
    let addr = SocketAddr::from(([0, 0, 0, 0], config.app.port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

/// Wait for Ctrl+C or SIGTERM for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix
            ::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("Ctrl+C received, shutting down..."); },
        _ = terminate => { tracing::info!("SIGTERM received, shutting down..."); },
    }
}
