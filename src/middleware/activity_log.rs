use sqlx::MySqlPool;
use uuid::Uuid;

/// Log user activity for security audit trail.
pub async fn log_activity(
    pool: &MySqlPool,
    user_id: &str,
    action: &str, // create | read | update | delete
    feature: &str, // e.g., "mood_tracker", "chat", "notes"
    entity_type: &str,
    entity_id: Option<&str>,
    details: Option<&str>,
    ip_address: Option<&str>
) {
    let id = Uuid::new_v4().to_string();

    let result = sqlx
        ::query(
            r#"
        INSERT INTO user_activity_logs (id, user_id, action, feature, entity_type, entity_id, details, ip_address)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
        )
        .bind(&id)
        .bind(user_id)
        .bind(action)
        .bind(feature)
        .bind(entity_type)
        .bind(entity_id)
        .bind(details)
        .bind(ip_address)
        .execute(pool).await;

    if let Err(e) = result {
        tracing::error!("Failed to log activity: {}", e);
    }
}

/// Log authentication events.
pub async fn log_auth_event(
    pool: &MySqlPool,
    user_id: &str,
    action: &str, // login | logout | token_refresh | email_verify | verification_sent
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    success: bool,
    failure_reason: Option<&str>
) {
    let id = Uuid::new_v4().to_string();

    let result = sqlx
        ::query(
            r#"
        INSERT INTO user_auth_logs (id, user_id, action, ip_address, user_agent, success, failure_reason)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
        )
        .bind(&id)
        .bind(user_id)
        .bind(action)
        .bind(ip_address)
        .bind(user_agent)
        .bind(success)
        .bind(failure_reason)
        .execute(pool).await;

    if let Err(e) = result {
        tracing::error!("Failed to log auth event: {}", e);
    }
}
