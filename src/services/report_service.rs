use chrono::NaiveDate;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{AppError, Result},
    models::mental::{
        AnalyticsSummaryRow, GenerateReportRequest, MentalReportResponse, MentalReportRow,
    },
    services::{email_service::EmailService, gemini_service::GeminiService},
};

pub struct ReportService;

impl ReportService {
    /// Generate a mental health report (weekly/monthly/quarterly/custom).
    pub async fn generate_report(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        email_service: &EmailService,
        user_id: &str,
        user_name: &str,
        user_email: &str,
        encryption_salt: &str,
        req: &GenerateReportRequest,
        trigger: &str, // automatic | manual | agentic
    ) -> Result<MentalReportResponse> {
        let report_type = req.report_type.as_deref().unwrap_or("weekly");
        let send_email = req.send_email.unwrap_or(false);

        let (period_start, period_end) = Self::resolve_period(report_type, req)?;

        let mood_entries: Vec<(
            chrono::NaiveDate,
            i8,
            Option<i8>,
            Option<i8>,
            Option<i8>,
            Option<String>,
        )> = sqlx::query_as(
            r#"SELECT entry_date, mood_score, energy_level, anxiety_level, stress_level, appetite
                   FROM mood_entries
                   WHERE user_id = ? AND entry_date BETWEEN ? AND ?
                   ORDER BY entry_date ASC"#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let analytics: Vec<AnalyticsSummaryRow> = sqlx::query_as(
            r#"SELECT * FROM mental_analytics_summaries
               WHERE user_id = ? AND period_start >= ? AND period_end <= ?
               ORDER BY created_at DESC LIMIT 3"#,
        )
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        let mood_json = serde_json::to_string(
            &mood_entries
                .iter()
                .map(|(date, mood, energy, anxiety, stress, appetite)| {
                    serde_json::json!({
                        "date": date,
                        "mood_score": mood,
                        "energy_level": energy,
                        "anxiety_level": anxiety,
                        "stress_level": stress,
                        "appetite": appetite,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap_or_default();

        let analytics_context = if !analytics.is_empty() {
            let decrypted: Vec<String> = analytics
                .iter()
                .filter_map(|a| {
                    crypto
                        .decrypt(&a.summary_enc, encryption_salt)
                        .ok()
                        .map(|s| format!("[{} {}]: {}", a.period_type, a.period_start, s))
                })
                .collect();
            decrypted.join("\n")
        } else {
            "No previous analytics available.".to_string()
        };

        let prompt = format!(
            r#"You are a professional mental health report generator. Create a comprehensive {report_type} report for user "{user_name}".

Period: {start} to {end}
Total mood entries: {count}

MOOD DATA:
{mood}

PREVIOUS ANALYTICS:
{analytics}

Generate a thorough report as JSON with this structure:
{{
  "title": "A descriptive title for this {report_type} report",
  "content": "A comprehensive multi-paragraph report covering:\n1. Overview of the period\n2. Mood patterns and trends\n3. Sleep and energy analysis\n4. Stress and anxiety observations\n5. Notable changes or events",
  "ai_analysis": "Deep analytical insights including:\n1. Pattern recognition across the data\n2. Correlation between different metrics\n3. Comparison with previous periods if available\n4. Risk assessment and early warning signs",
  "recommendations": "Specific actionable recommendations:\n1. Immediate actions (this week)\n2. Short-term goals (next 2-4 weeks)\n3. Lifestyle adjustments\n4. When to seek professional help\n5. Coping strategies to try",
  "mood_trend": "improving|stable|declining",
  "risk_level": "low|moderate|high|severe",
  "avg_mood": <number 1-10>
}}

Be empathetic, evidence-based, and constructive. Do NOT diagnose. Return ONLY valid JSON."#,
            report_type = report_type,
            user_name = user_name,
            start = period_start,
            end = period_end,
            count = mood_entries.len(),
            mood = mood_json,
            analytics = analytics_context
        );

        let ai_raw = gemini
            .generate_with_system(
                &prompt,
                Some(
                    "You are a professional mental health report writer. Always respond with valid JSON."
                ),
                0.4,
                8192
            ).await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let ai_json: serde_json::Value = Self::parse_ai_json(&ai_raw)?;

        let title = ai_json["title"]
            .as_str()
            .unwrap_or("Mental Health Report")
            .to_string();
        let content = ai_json["content"].as_str().unwrap_or("").to_string();
        let ai_analysis = ai_json["ai_analysis"].as_str().unwrap_or("").to_string();
        let recommendations = ai_json["recommendations"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let mood_trend = ai_json["mood_trend"]
            .as_str()
            .unwrap_or("stable")
            .to_string();
        let risk_level = ai_json["risk_level"].as_str().unwrap_or("low").to_string();
        let avg_mood = ai_json["avg_mood"].as_f64().map(|v| v as f32);

        let content_enc = crypto.encrypt(&content, encryption_salt)?;
        let analysis_enc = crypto.encrypt(&ai_analysis, encryption_salt)?;
        let recs_enc = crypto.encrypt(&recommendations, encryption_salt)?;

        let id = Uuid::new_v4().to_string();
        let mut status = "generated".to_string();

        sqlx::query(
            r#"
            INSERT INTO mental_reports
            (id, user_id, report_type, period_start, period_end, title, content_enc,
             ai_analysis_enc, recommendations_enc, status, sent_via_email, trigger_type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(report_type)
        .bind(period_start)
        .bind(period_end)
        .bind(&title)
        .bind(&content_enc)
        .bind(&analysis_enc)
        .bind(&recs_enc)
        .bind(&status)
        .bind(send_email)
        .bind(trigger)
        .execute(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        if send_email {
            let email_result = email_service
                .send_report_email(
                    user_email,
                    user_name,
                    report_type,
                    &period_start.to_string(),
                    &period_end.to_string(),
                    &content,
                    &recommendations,
                    &mood_trend,
                    avg_mood,
                    &risk_level,
                )
                .await;

            match email_result {
                Ok(_) => {
                    status = "sent".to_string();
                    sqlx::query(
                        "UPDATE mental_reports SET status = 'sent', sent_at = NOW() WHERE id = ?",
                    )
                    .bind(&id)
                    .execute(pool)
                    .await
                    .ok();
                }
                Err(e) => {
                    tracing::error!("Failed to send report email: {:?}", e);
                    sqlx::query("UPDATE mental_reports SET status = 'failed' WHERE id = ?")
                        .bind(&id)
                        .execute(pool)
                        .await
                        .ok();
                    status = "failed".to_string();
                }
            }
        }

        Ok(MentalReportResponse {
            id,
            report_type: report_type.to_string(),
            period_start: period_start.to_string(),
            period_end: period_end.to_string(),
            title,
            content,
            ai_analysis,
            recommendations,
            status,
            sent_via_email: send_email,
            trigger_type: trigger.to_string(),
            created_at: chrono::Local::now().naive_local().to_string(),
        })
    }

    pub async fn get_reports(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        limit: i64,
    ) -> Result<Vec<MentalReportResponse>> {
        let rows: Vec<MentalReportRow> = sqlx::query_as(
            r#"SELECT * FROM mental_reports
               WHERE user_id = ?
               ORDER BY created_at DESC
               LIMIT ?"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::DatabaseError)?;

        rows.iter()
            .map(|r| Self::decrypt_row(crypto, r, encryption_salt))
            .collect()
    }

    pub async fn get_report(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        report_id: &str,
        encryption_salt: &str,
    ) -> Result<MentalReportResponse> {
        let row: MentalReportRow =
            sqlx::query_as("SELECT * FROM mental_reports WHERE id = ? AND user_id = ?")
                .bind(report_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::DatabaseError)?
                .ok_or_else(|| AppError::NotFound("Report not found".into()))?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    fn decrypt_row(
        crypto: &CryptoService,
        row: &MentalReportRow,
        salt: &str,
    ) -> Result<MentalReportResponse> {
        Ok(MentalReportResponse {
            id: row.id.clone(),
            report_type: row.report_type.clone(),
            period_start: row.period_start.to_string(),
            period_end: row.period_end.to_string(),
            title: row.title.clone(),
            content: crypto.decrypt(&row.content_enc, salt)?,
            ai_analysis: crypto.decrypt(&row.ai_analysis_enc, salt)?,
            recommendations: crypto.decrypt(&row.recommendations_enc, salt)?,
            status: row.status.clone(),
            sent_via_email: row.sent_via_email,
            trigger_type: row.trigger_type.clone(),
            created_at: row.created_at.to_string(),
        })
    }

    fn resolve_period(
        report_type: &str,
        req: &GenerateReportRequest,
    ) -> Result<(NaiveDate, NaiveDate)> {
        let today = chrono::Local::now().date_naive();

        match report_type {
            "custom" => {
                let start = req
                    .period_start
                    .as_deref()
                    .ok_or_else(|| {
                        AppError::ValidationError("period_start required for custom report".into())
                    })
                    .and_then(|s| {
                        NaiveDate::parse_from_str(s, "%Y-%m-%d")
                            .map_err(|_| AppError::ValidationError("Invalid date format".into()))
                    })?;
                let end = req
                    .period_end
                    .as_deref()
                    .ok_or_else(|| {
                        AppError::ValidationError("period_end required for custom report".into())
                    })
                    .and_then(|s| {
                        NaiveDate::parse_from_str(s, "%Y-%m-%d")
                            .map_err(|_| AppError::ValidationError("Invalid date format".into()))
                    })?;
                Ok((start, end))
            }
            "monthly" => Ok((today - chrono::Duration::days(29), today)),
            "quarterly" => Ok((today - chrono::Duration::days(89), today)),
            "yearly" => Ok((today - chrono::Duration::days(364), today)),
            _ => Ok((today - chrono::Duration::days(6), today)), // weekly default
        }
    }

    fn parse_ai_json(raw: &str) -> Result<serde_json::Value> {
        let trimmed = raw.trim();
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            return Ok(v);
        }
        let cleaned = trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str::<serde_json::Value>(cleaned).map_err(|e| {
            tracing::error!("Failed to parse AI report JSON: {}. Raw: {}", e, raw);
            AppError::InternalError(anyhow::anyhow!("Failed to parse AI response"))
        })
    }
}
