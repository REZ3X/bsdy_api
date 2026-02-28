use chrono::{ NaiveDate, NaiveDateTime };
use serde::{ Deserialize, Serialize };

// ── Mental Characteristics (Baseline Assessment) ────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MentalCharacteristicRow {
    pub id: String,
    pub user_id: String,
    pub risk_level: String,
    pub assessment_version: i32,
    pub family_background_enc: Option<String>,
    pub stress_level_enc: String,
    pub anxiety_level_enc: String,
    pub depression_level_enc: String,
    pub sleep_quality_enc: String,
    pub social_support_enc: String,
    pub coping_style_enc: String,
    pub personality_traits_enc: String,
    pub mental_health_history_enc: String,
    pub current_medications_enc: Option<String>,
    pub therapy_status_enc: String,
    pub additional_notes_enc: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct BaselineAssessmentRequest {
    pub birth: String, // YYYY-MM-DD
    pub family_background: Option<String>, // optional
    pub stress_level: String, // "low" | "moderate" | "high" | "severe"
    pub anxiety_level: String,
    pub depression_level: String,
    pub sleep_quality: String,
    pub social_support: String, // "none" | "low" | "moderate" | "strong"
    pub coping_style: String, // "avoidant" | "problem_focused" | "emotion_focused" | "social_support_seeking"
    pub personality_traits: String, // JSON array of traits
    pub mental_health_history: String, // narrative
    pub current_medications: Option<String>,
    pub therapy_status: String, // "none" | "considering" | "active" | "past"
    pub additional_notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BaselineAssessmentResponse {
    pub id: String,
    pub user_id: String,
    pub risk_level: String,
    pub family_background: Option<String>,
    pub stress_level: String,
    pub anxiety_level: String,
    pub depression_level: String,
    pub sleep_quality: String,
    pub social_support: String,
    pub coping_style: String,
    pub personality_traits: String,
    pub mental_health_history: String,
    pub current_medications: Option<String>,
    pub therapy_status: String,
    pub additional_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBaselineRequest {
    pub family_background: Option<String>,
    pub stress_level: Option<String>,
    pub anxiety_level: Option<String>,
    pub depression_level: Option<String>,
    pub sleep_quality: Option<String>,
    pub social_support: Option<String>,
    pub coping_style: Option<String>,
    pub personality_traits: Option<String>,
    pub mental_health_history: Option<String>,
    pub current_medications: Option<String>,
    pub therapy_status: Option<String>,
    pub additional_notes: Option<String>,
}

// ── Mood Entry ──────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MoodEntryRow {
    pub id: String,
    pub user_id: String,
    pub entry_date: NaiveDate,
    pub mood_score: i8,
    pub energy_level: Option<i8>,
    pub anxiety_level: Option<i8>,
    pub stress_level: Option<i8>,
    pub sleep_hours: Option<rust_decimal::Decimal>,
    pub sleep_quality: Option<i8>,
    pub appetite: Option<String>,
    pub social_interaction: Option<bool>,
    pub exercise_done: Option<bool>,
    pub notes_enc: Option<String>,
    pub triggers_enc: Option<String>,
    pub activities_enc: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateMoodEntryRequest {
    pub mood_score: i8, // 1-10
    pub energy_level: Option<i8>,
    pub anxiety_level: Option<i8>,
    pub stress_level: Option<i8>,
    pub sleep_hours: Option<f32>,
    pub sleep_quality: Option<i8>,
    pub appetite: Option<String>,
    pub social_interaction: Option<bool>,
    pub exercise_done: Option<bool>,
    pub notes: Option<String>,
    pub triggers: Option<String>, // JSON array
    pub activities: Option<String>, // JSON array
}

#[derive(Debug, Serialize)]
pub struct MoodEntryResponse {
    pub id: String,
    pub entry_date: String,
    pub mood_score: i8,
    pub energy_level: Option<i8>,
    pub anxiety_level: Option<i8>,
    pub stress_level: Option<i8>,
    pub sleep_hours: Option<f32>,
    pub sleep_quality: Option<i8>,
    pub appetite: Option<String>,
    pub social_interaction: Option<bool>,
    pub exercise_done: Option<bool>,
    pub notes: Option<String>,
    pub triggers: Option<String>,
    pub activities: Option<String>,
    pub created_at: String,
}

// ── Analytics Summary ───────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalyticsSummaryRow {
    pub id: String,
    pub user_id: String,
    pub period_type: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub summary_enc: String,
    pub insights_enc: String,
    pub recommendations_enc: String,
    pub overall_mood_trend: String,
    pub avg_mood_score: Option<rust_decimal::Decimal>,
    pub risk_level: String,
    pub generated_by: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsSummaryResponse {
    pub id: String,
    pub period_type: String,
    pub period_start: String,
    pub period_end: String,
    pub summary: String,
    pub insights: String,
    pub recommendations: String,
    pub overall_mood_trend: String,
    pub avg_mood_score: Option<f32>,
    pub risk_level: String,
    pub generated_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateAnalyticsRequest {
    pub period_type: Option<String>, // weekly | monthly | quarterly
}

// ── Mental Report ───────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MentalReportRow {
    pub id: String,
    pub user_id: String,
    pub report_type: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub title: String,
    pub content_enc: String,
    pub ai_analysis_enc: String,
    pub recommendations_enc: String,
    pub status: String,
    pub sent_via_email: bool,
    pub sent_at: Option<NaiveDateTime>,
    pub trigger_type: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct MentalReportResponse {
    pub id: String,
    pub report_type: String,
    pub period_start: String,
    pub period_end: String,
    pub title: String,
    pub content: String,
    pub ai_analysis: String,
    pub recommendations: String,
    pub status: String,
    pub sent_via_email: bool,
    pub trigger_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: Option<String>, // weekly | monthly | quarterly | custom
    pub period_start: Option<String>, // custom only
    pub period_end: Option<String>,
    pub send_email: Option<bool>,
}
