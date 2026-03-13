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
    pub async fn start(
        pool: MySqlPool,
        config: Arc<Config>,
        crypto: Arc<CryptoService>,
        gemini: Arc<GeminiService>,
        email: Arc<EmailService>
    ) -> Result<JobScheduler> {
        let sched = JobScheduler::new().await.map_err(|e| AppError::InternalError(e.into()))?;

        // ── Weekly ──────────────────────────────────────────
        Self::register_job(
            &sched,
            &config.scheduler.weekly_report_cron,
            "weekly",
            pool.clone(),
            crypto.clone(),
            gemini.clone(),
            email.clone()
        ).await?;

        // ── Monthly ─────────────────────────────────────────
        Self::register_job(
            &sched,
            &config.scheduler.monthly_report_cron,
            "monthly",
            pool.clone(),
            crypto.clone(),
            gemini.clone(),
            email.clone()
        ).await?;

        // ── Yearly ──────────────────────────────────────────
        Self::register_job(
            &sched,
            &config.scheduler.yearly_report_cron,
            "yearly",
            pool.clone(),
            crypto.clone(),
            gemini.clone(),
            email.clone()
        ).await?;

        sched.start().await.map_err(|e| AppError::InternalError(e.into()))?;

        tracing::info!("Background scheduler started (weekly + monthly + yearly)");
        Ok(sched)
    }

    async fn register_job(
        sched: &JobScheduler,
        cron_expr: &str,
        report_type: &str,
        pool: MySqlPool,
        crypto: Arc<CryptoService>,
        gemini: Arc<GeminiService>,
        email: Arc<EmailService>
    ) -> Result<()> {
        let rtype = report_type.to_string();
        let cron = cron_expr.to_string();
        tracing::info!("Registering {} report cron: {}", rtype, cron);

        let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
            let pool = pool.clone();
            let crypto = crypto.clone();
            let gemini = gemini.clone();
            let email = email.clone();
            let rtype = rtype.clone();

            Box::pin(async move {
                tracing::info!("Running scheduled {} report generation", rtype);

                if
                    let Err(e) = Self::generate_reports_for_all(
                        &pool,
                        &crypto,
                        &gemini,
                        &email,
                        &rtype
                    ).await
                {
                    tracing::error!("Scheduled {} report generation failed: {:?}", rtype, e);
                }
            })
        }).map_err(|e| AppError::InternalError(e.into()))?;

        sched.add(job).await.map_err(|e| AppError::InternalError(e.into()))?;
        Ok(())
    }

    async fn generate_reports_for_all(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email: &EmailService,
        report_type: &str
    ) -> Result<()> {
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

        tracing::info!("Found {} eligible users for {} reports", users.len(), report_type);

        let mut success_count = 0u32;
        let mut error_count = 0u32;

        for (user_id, user_name, user_email, encryption_salt) in &users {
            let req = GenerateReportRequest {
                report_type: Some(report_type.to_string()),
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
                        "{} report generated for user {} (report: {})",
                        report_type,
                        user_id,
                        report.id
                    );
                    success_count += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to generate {} report for user {}: {:?}",
                        report_type,
                        user_id,
                        e
                    );
                    error_count += 1;
                }
            }
        }

        let task_id = uuid::Uuid::new_v4().to_string();
        let task_type = format!("{}_report", report_type);
        let result_json =
            serde_json::json!({
            "total_users": users.len(),
            "success": success_count,
            "errors": error_count,
        });

        sqlx::query(
            r#"INSERT INTO scheduled_tasks (id, task_type, status, result, started_at, completed_at)
               VALUES (?, ?, ?, ?, NOW(), NOW())"#
        )
            .bind(&task_id)
            .bind(&task_type)
            .bind(if error_count == 0 { "completed" } else { "partial" })
            .bind(result_json.to_string())
            .execute(pool).await
            .ok();

        tracing::info!(
            "{} report generation complete: {} success, {} errors",
            report_type,
            success_count,
            error_count
        );

        Ok(())
    }
}
