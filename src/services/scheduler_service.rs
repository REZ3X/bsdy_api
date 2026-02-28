use std::sync::Arc;

use sqlx::MySqlPool;
use tokio_cron_scheduler::{ Job, JobScheduler };

use crate::{
    config::Config,
    crypto::CryptoService,
    error::{ AppError, Result },
    models::mental::GenerateReportRequest,
    services::{
        email_service::EmailService,
        gemini_service::GeminiService,
        report_service::ReportService,
    },
};

pub struct SchedulerService;

impl SchedulerService {
    /// Start the background scheduler that runs weekly report generation.
    pub async fn start(
        pool: MySqlPool,
        config: Arc<Config>,
        crypto: Arc<CryptoService>,
        gemini: Arc<GeminiService>,
        email: Arc<EmailService>
    ) -> Result<JobScheduler> {
        let sched = JobScheduler::new().await.map_err(|e| AppError::InternalError(e.into()))?;

        let cron_expr = config.scheduler.weekly_report_cron.clone();
        tracing::info!("Registering weekly report cron: {}", cron_expr);

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let pool = pool.clone();
            let crypto = crypto.clone();
            let gemini = gemini.clone();
            let email = email.clone();

            Box::pin(async move {
                tracing::info!("Running scheduled weekly report generation");

                if
                    let Err(e) = Self::generate_weekly_reports(
                        &pool,
                        &crypto,
                        &gemini,
                        &email
                    ).await
                {
                    tracing::error!("Scheduled report generation failed: {:?}", e);
                }
            })
        }).map_err(|e| AppError::InternalError(e.into()))?;

        sched.add(job).await.map_err(|e| AppError::InternalError(e.into()))?;

        sched.start().await.map_err(|e| AppError::InternalError(e.into()))?;

        tracing::info!("Background scheduler started");
        Ok(sched)
    }

    /// Generate weekly reports for all eligible users (verified + onboarded).
    async fn generate_weekly_reports(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email: &EmailService
    ) -> Result<()> {
        // Find users who are verified and have completed onboarding
        let users: Vec<(String, String, String, String)> = sqlx
            ::query_as(
                r#"SELECT u.id, u.name, u.email, u.encryption_salt
               FROM users u
               INNER JOIN mental_characteristics mc ON mc.user_id = u.id
               WHERE u.is_verified = true
                 AND u.encryption_salt IS NOT NULL
                 AND u.encryption_salt != ''
               ORDER BY u.created_at"#
            )
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        tracing::info!("Found {} eligible users for weekly reports", users.len());

        let mut success_count = 0u32;
        let mut error_count = 0u32;

        for (user_id, user_name, user_email, encryption_salt) in &users {
            let req = GenerateReportRequest {
                report_type: Some("weekly".to_string()),
                period_start: None,
                period_end: None,
                send_email: Some(true),
            };

            match
                ReportService::generate_report(
                    pool,
                    crypto,
                    gemini,
                    email,
                    user_id,
                    user_name,
                    user_email,
                    encryption_salt,
                    &req,
                    "automatic"
                ).await
            {
                Ok(report) => {
                    tracing::info!(
                        "Weekly report generated for user {} (report: {})",
                        user_id,
                        report.id
                    );
                    success_count += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to generate weekly report for user {}: {:?}",
                        user_id,
                        e
                    );
                    error_count += 1;
                }
            }
        }

        // Log the scheduled task result
        let task_id = uuid::Uuid::new_v4().to_string();
        let result_json =
            serde_json::json!({
            "total_users": users.len(),
            "success": success_count,
            "errors": error_count,
        });

        sqlx::query(
            r#"INSERT INTO scheduled_tasks (id, task_type, status, result, started_at, completed_at)
               VALUES (?, 'weekly_report', ?, ?, NOW(), NOW())"#
        )
            .bind(&task_id)
            .bind(if error_count == 0 { "completed" } else { "partial" })
            .bind(result_json.to_string())
            .execute(pool).await
            .ok();

        tracing::info!(
            "Weekly report generation complete: {} success, {} errors",
            success_count,
            error_count
        );

        Ok(())
    }
}
