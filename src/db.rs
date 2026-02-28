use sqlx::mysql::{ MySqlPool, MySqlPoolOptions };

use crate::config::DatabaseConfig;

pub async fn create_pool(config: &DatabaseConfig) -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&config.url).await?;

    tracing::info!("Database connection pool established");
    Ok(pool)
}

/// Run SQL migration file against the pool.
pub async fn run_migrations(pool: &MySqlPool) -> Result<(), anyhow::Error> {
    let migration_sql = include_str!("../migrations/001_initial_schema.sql");

    // Split by statement separator and execute each
    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("SET ") {
            continue;
        }
        sqlx::query(trimmed)
            .execute(pool).await
            .map_err(|e| {
                tracing::warn!("Migration statement skipped or failed (may already exist): {}", e);
                e
            })
            .ok();
    }

    tracing::info!("Database migrations completed");
    Ok(())
}
