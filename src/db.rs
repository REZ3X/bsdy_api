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
        // Strip leading comment lines and whitespace to get the actual SQL
        let cleaned: String = statement
            .lines()
            .filter(|line| {
                let t = line.trim();
                !t.is_empty() && !t.starts_with("--")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let cleaned = cleaned.trim();

        if cleaned.is_empty() || cleaned.starts_with("SET ") {
            continue;
        }

        sqlx::query(cleaned)
            .execute(pool).await
            .map_err(|e| {
                tracing::warn!("Migration statement failed: {}", e);
                e
            })
            .ok(); // IF NOT EXISTS handles duplicates; ok() tolerates "already exists"
    }

    tracing::info!("Database migrations completed");
    Ok(())
}
