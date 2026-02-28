use chrono::NaiveDate;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::mental::{ CreateMoodEntryRequest, MoodEntryResponse, MoodEntryRow },
};

pub struct MoodService;

impl MoodService {
    /// Create a mood entry. Only one per day; if exists, update.
    pub async fn upsert_mood(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        req: &CreateMoodEntryRequest
    ) -> Result<MoodEntryResponse> {
        if req.mood_score < 1 || req.mood_score > 10 {
            return Err(AppError::ValidationError("mood_score must be between 1 and 10".into()));
        }

        let today = chrono::Local::now().date_naive();
        let notes_enc = crypto.encrypt_optional(req.notes.as_deref(), encryption_salt)?;
        let triggers_enc = crypto.encrypt_optional(req.triggers.as_deref(), encryption_salt)?;
        let activities_enc = crypto.encrypt_optional(req.activities.as_deref(), encryption_salt)?;

        // Check if today's entry exists
        let existing: Option<(String,)> = sqlx
            ::query_as("SELECT id FROM mood_entries WHERE user_id = ? AND entry_date = ?")
            .bind(user_id)
            .bind(today)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?;

        let id = if let Some((existing_id,)) = existing {
            // Update
            sqlx
                ::query(
                    r#"
                UPDATE mood_entries SET
                    mood_score = ?, energy_level = ?, anxiety_level = ?, stress_level = ?,
                    sleep_hours = ?, sleep_quality = ?, appetite = ?,
                    social_interaction = ?, exercise_done = ?,
                    notes_enc = ?, triggers_enc = ?, activities_enc = ?
                WHERE id = ?
            "#
                )
                .bind(req.mood_score)
                .bind(req.energy_level)
                .bind(req.anxiety_level)
                .bind(req.stress_level)
                .bind(req.sleep_hours)
                .bind(req.sleep_quality)
                .bind(req.appetite.as_deref())
                .bind(req.social_interaction)
                .bind(req.exercise_done)
                .bind(notes_enc.as_deref())
                .bind(triggers_enc.as_deref())
                .bind(activities_enc.as_deref())
                .bind(&existing_id)
                .execute(pool).await
                .map_err(AppError::DatabaseError)?;
            existing_id
        } else {
            // Insert
            let id = Uuid::new_v4().to_string();
            sqlx
                ::query(
                    r#"
                INSERT INTO mood_entries
                (id, user_id, entry_date, mood_score, energy_level, anxiety_level, stress_level,
                 sleep_hours, sleep_quality, appetite, social_interaction, exercise_done,
                 notes_enc, triggers_enc, activities_enc)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
                )
                .bind(&id)
                .bind(user_id)
                .bind(today)
                .bind(req.mood_score)
                .bind(req.energy_level)
                .bind(req.anxiety_level)
                .bind(req.stress_level)
                .bind(req.sleep_hours)
                .bind(req.sleep_quality)
                .bind(req.appetite.as_deref())
                .bind(req.social_interaction)
                .bind(req.exercise_done)
                .bind(notes_enc.as_deref())
                .bind(triggers_enc.as_deref())
                .bind(activities_enc.as_deref())
                .execute(pool).await
                .map_err(AppError::DatabaseError)?;
            id
        };

        let row: MoodEntryRow = sqlx
            ::query_as("SELECT * FROM mood_entries WHERE id = ?")
            .bind(&id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Get mood entries for a date range.
    pub async fn get_mood_entries(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        limit: Option<u32>
    ) -> Result<Vec<MoodEntryResponse>> {
        let limit = limit.unwrap_or(30).min(90) as i64;
        let now = chrono::Local::now().date_naive();
        let to_date = to.unwrap_or(now);
        let from_date = from.unwrap_or_else(|| to_date - chrono::Duration::days(29));

        let rows: Vec<MoodEntryRow> = sqlx
            ::query_as(
                r#"SELECT * FROM mood_entries
               WHERE user_id = ? AND entry_date BETWEEN ? AND ?
               ORDER BY entry_date DESC
               LIMIT ?"#
            )
            .bind(user_id)
            .bind(from_date)
            .bind(to_date)
            .bind(limit)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        rows.iter()
            .map(|r| Self::decrypt_row(crypto, r, encryption_salt))
            .collect()
    }

    /// Get today's mood entry.
    pub async fn get_today(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str
    ) -> Result<Option<MoodEntryResponse>> {
        let today = chrono::Local::now().date_naive();

        let row: Option<MoodEntryRow> = sqlx
            ::query_as("SELECT * FROM mood_entries WHERE user_id = ? AND entry_date = ?")
            .bind(user_id)
            .bind(today)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?;

        row.map(|r| Self::decrypt_row(crypto, &r, encryption_salt)).transpose()
    }

    fn decrypt_row(
        crypto: &CryptoService,
        row: &MoodEntryRow,
        salt: &str
    ) -> Result<MoodEntryResponse> {
        Ok(MoodEntryResponse {
            id: row.id.clone(),
            entry_date: row.entry_date.to_string(),
            mood_score: row.mood_score,
            energy_level: row.energy_level,
            anxiety_level: row.anxiety_level,
            stress_level: row.stress_level,
            sleep_hours: row.sleep_hours
                .as_ref()
                .map(|d| { d.to_string().parse::<f32>().unwrap_or(0.0) }),
            sleep_quality: row.sleep_quality,
            appetite: row.appetite.clone(),
            social_interaction: row.social_interaction,
            exercise_done: row.exercise_done,
            notes: crypto.decrypt_optional(row.notes_enc.as_deref(), salt)?,
            triggers: crypto.decrypt_optional(row.triggers_enc.as_deref(), salt)?,
            activities: crypto.decrypt_optional(row.activities_enc.as_deref(), salt)?,
            created_at: row.created_at.to_string(),
        })
    }
}
