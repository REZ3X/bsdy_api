use chrono::NaiveDate;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::mental::{ AnalyticsSummaryResponse, AnalyticsSummaryRow, MentalCharacteristicRow },
    services::gemini_service::GeminiService,
};

pub struct AnalyticsService;

impl AnalyticsService {
    /// Generate an analytics summary for the user using AI.
    pub async fn generate_summary(
        pool: &MySqlPool,
        crypto: &CryptoService,
        gemini: &GeminiService,
        user_id: &str,
        user_name: &str,
        encryption_salt: &str,
        period: &str, // weekly | monthly | quarterly
        trigger: &str // automatic | manual
    ) -> Result<AnalyticsSummaryResponse> {
        let (period_start, period_end) = Self::period_dates(period);

        // Gather mood entries for the period
        let mood_rows: Vec<(String, i8, Option<i8>, Option<i8>, Option<i8>)> = sqlx
            ::query_as(
                r#"SELECT id, mood_score, energy_level, anxiety_level, stress_level
               FROM mood_entries
               WHERE user_id = ? AND entry_date BETWEEN ? AND ?
               ORDER BY entry_date ASC"#
            )
            .bind(user_id)
            .bind(period_start)
            .bind(period_end)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        if mood_rows.is_empty() {
            return Err(
                AppError::BadRequest(
                    "No mood data found for this period. Please log at least one mood entry.".into()
                )
            );
        }

        let mood_json = serde_json
            ::to_string(
                &mood_rows
                    .iter()
                    .map(|(_id, mood, energy, anxiety, stress)| {
                        serde_json::json!({
                "mood_score": mood,
                "energy_level": energy,
                "anxiety_level": anxiety,
                "stress_level": stress,
            })
                    })
                    .collect::<Vec<_>>()
            )
            .unwrap_or_default();

        // Get baseline for context
        let baseline_row: Option<MentalCharacteristicRow> = sqlx
            ::query_as("SELECT * FROM mental_characteristics WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?;

        let baseline_json = if let Some(row) = baseline_row {
            let stress = crypto.decrypt(&row.stress_level_enc, encryption_salt).unwrap_or_default();
            let anxiety = crypto
                .decrypt(&row.anxiety_level_enc, encryption_salt)
                .unwrap_or_default();
            let depression = crypto
                .decrypt(&row.depression_level_enc, encryption_salt)
                .unwrap_or_default();
            let therapy = crypto
                .decrypt(&row.therapy_status_enc, encryption_salt)
                .unwrap_or_default();
            serde_json
                ::to_string(
                    &serde_json::json!({
                "risk_level": row.risk_level,
                "baseline_stress": stress,
                "baseline_anxiety": anxiety,
                "baseline_depression": depression,
                "therapy_status": therapy,
            })
                )
                .unwrap_or_default()
        } else {
            "{}".to_string()
        };

        // Call Gemini
        let ai_response = gemini
            .analyze_mood_data(user_name, &mood_json, &baseline_json, period).await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Parse AI response
        let ai_json: serde_json::Value = serde_json
            ::from_str(&ai_response.trim())
            .unwrap_or_else(|_| {
                // Try to extract JSON from markdown code block
                let cleaned = ai_response
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                serde_json
                    ::from_str(cleaned)
                    .unwrap_or_else(
                        |_|
                            serde_json::json!({
                "summary": ai_response,
                "insights": "Unable to parse AI insights.",
                "recommendations": "Please try again.",
                "overall_mood_trend": "stable",
                "risk_level": "low",
                "avg_mood_score": 5.0
            })
                    )
            });

        let summary = ai_json["summary"].as_str().unwrap_or("").to_string();
        let insights = ai_json["insights"].as_str().unwrap_or("").to_string();
        let recommendations = ai_json["recommendations"].as_str().unwrap_or("").to_string();
        let mood_trend = ai_json["overall_mood_trend"].as_str().unwrap_or("stable").to_string();
        let risk_level = ai_json["risk_level"].as_str().unwrap_or("low").to_string();
        let avg_mood = ai_json["avg_mood_score"].as_f64().map(|v| v as f32);

        // Encrypt
        let summary_enc = crypto.encrypt(&summary, encryption_salt)?;
        let insights_enc = crypto.encrypt(&insights, encryption_salt)?;
        let recs_enc = crypto.encrypt(&recommendations, encryption_salt)?;

        let id = Uuid::new_v4().to_string();

        sqlx
            ::query(
                r#"
            INSERT INTO mental_analytics_summaries
            (id, user_id, period_type, period_start, period_end, summary_enc, insights_enc,
             recommendations_enc, overall_mood_trend, avg_mood_score, risk_level, generated_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
            )
            .bind(&id)
            .bind(user_id)
            .bind(period)
            .bind(period_start)
            .bind(period_end)
            .bind(&summary_enc)
            .bind(&insights_enc)
            .bind(&recs_enc)
            .bind(&mood_trend)
            .bind(avg_mood)
            .bind(&risk_level)
            .bind(trigger)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: AnalyticsSummaryRow = sqlx
            ::query_as("SELECT * FROM mental_analytics_summaries WHERE id = ?")
            .bind(&id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Get list of analytics summaries for a user.
    pub async fn get_summaries(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        limit: i64
    ) -> Result<Vec<AnalyticsSummaryResponse>> {
        let rows: Vec<AnalyticsSummaryRow> = sqlx
            ::query_as(
                r#"SELECT * FROM mental_analytics_summaries
               WHERE user_id = ?
               ORDER BY created_at DESC
               LIMIT ?"#
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        rows.iter()
            .map(|r| Self::decrypt_row(crypto, r, encryption_salt))
            .collect()
    }

    fn decrypt_row(
        crypto: &CryptoService,
        row: &AnalyticsSummaryRow,
        salt: &str
    ) -> Result<AnalyticsSummaryResponse> {
        Ok(AnalyticsSummaryResponse {
            id: row.id.clone(),
            period_type: row.period_type.clone(),
            period_start: row.period_start.to_string(),
            period_end: row.period_end.to_string(),
            summary: crypto.decrypt(&row.summary_enc, salt)?,
            insights: crypto.decrypt(&row.insights_enc, salt)?,
            recommendations: crypto.decrypt(&row.recommendations_enc, salt)?,
            overall_mood_trend: row.overall_mood_trend.clone(),
            avg_mood_score: row.avg_mood_score
                .as_ref()
                .map(|d| { d.to_string().parse::<f32>().unwrap_or(0.0) }),
            risk_level: row.risk_level.clone(),
            generated_by: row.generated_by.clone(),
            created_at: row.created_at.to_string(),
        })
    }

    pub fn period_dates(period: &str) -> (NaiveDate, NaiveDate) {
        let today = chrono::Local::now().date_naive();
        let start = match period {
            "monthly" => today - chrono::Duration::days(29),
            "quarterly" => today - chrono::Duration::days(89),
            _ => today - chrono::Duration::days(6), // weekly
        };
        (start, today)
    }
}
