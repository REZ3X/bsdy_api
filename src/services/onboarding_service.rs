use chrono::NaiveDate;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::mental::{
        BaselineAssessmentRequest,
        BaselineAssessmentResponse,
        MentalCharacteristicRow,
        UpdateBaselineRequest,
    },
};

pub struct OnboardingService;

impl OnboardingService {
    /// Save baseline assessment for a user. Returns error if already completed.
    pub async fn save_baseline(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        req: &BaselineAssessmentRequest
    ) -> Result<BaselineAssessmentResponse> {
        // Check if already exists
        let existing: Option<(String,)> = sqlx
            ::query_as("SELECT id FROM mental_characteristics WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?;

        if existing.is_some() {
            return Err(
                AppError::Conflict(
                    "Baseline assessment already completed. Use PATCH to update.".into()
                )
            );
        }

        // Compute simple risk level from inputs
        let risk_level = compute_risk_level(
            &req.stress_level,
            &req.anxiety_level,
            &req.depression_level
        );

        let id = Uuid::new_v4().to_string();
        let salt = encryption_salt;

        // Encrypt sensitive fields
        let family_enc = crypto.encrypt_optional(req.family_background.as_deref(), salt)?;
        let stress_enc = crypto.encrypt(&req.stress_level, salt)?;
        let anxiety_enc = crypto.encrypt(&req.anxiety_level, salt)?;
        let depression_enc = crypto.encrypt(&req.depression_level, salt)?;
        let sleep_enc = crypto.encrypt(&req.sleep_quality, salt)?;
        let social_enc = crypto.encrypt(&req.social_support, salt)?;
        let coping_enc = crypto.encrypt(&req.coping_style, salt)?;
        let personality_enc = crypto.encrypt(&req.personality_traits, salt)?;
        let history_enc = crypto.encrypt(&req.mental_health_history, salt)?;
        let medications_enc = crypto.encrypt_optional(req.current_medications.as_deref(), salt)?;
        let therapy_enc = crypto.encrypt(&req.therapy_status, salt)?;
        let notes_enc = crypto.encrypt_optional(req.additional_notes.as_deref(), salt)?;

        sqlx
            ::query(
                r#"
            INSERT INTO mental_characteristics
            (id, user_id, risk_level, assessment_version,
             family_background_enc, stress_level_enc, anxiety_level_enc, depression_level_enc,
             sleep_quality_enc, social_support_enc, coping_style_enc, personality_traits_enc,
             mental_health_history_enc, current_medications_enc, therapy_status_enc, additional_notes_enc)
            VALUES (?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
            )
            .bind(&id)
            .bind(user_id)
            .bind(&risk_level)
            .bind(family_enc.as_deref())
            .bind(&stress_enc)
            .bind(&anxiety_enc)
            .bind(&depression_enc)
            .bind(&sleep_enc)
            .bind(&social_enc)
            .bind(&coping_enc)
            .bind(&personality_enc)
            .bind(&history_enc)
            .bind(medications_enc.as_deref())
            .bind(&therapy_enc)
            .bind(notes_enc.as_deref())
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        // Update birth date and mark onboarding complete
        sqlx
            ::query(
                "UPDATE users SET birth = ?, onboarding_completed = TRUE, updated_at = NOW() WHERE id = ?"
            )
            .bind(&req.birth)
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: MentalCharacteristicRow = sqlx
            ::query_as("SELECT * FROM mental_characteristics WHERE id = ?")
            .bind(&id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &row, salt)
    }

    /// Retrieve the baseline for a user.
    pub async fn get_baseline(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str
    ) -> Result<BaselineAssessmentResponse> {
        let row: MentalCharacteristicRow = sqlx
            ::query_as("SELECT * FROM mental_characteristics WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Baseline assessment not found".into()))?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Partially update the baseline fields.
    pub async fn update_baseline(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        req: &UpdateBaselineRequest
    ) -> Result<BaselineAssessmentResponse> {
        let existing: MentalCharacteristicRow = sqlx
            ::query_as("SELECT * FROM mental_characteristics WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(||
                AppError::NotFound(
                    "Baseline assessment not found. Complete onboarding first.".into()
                )
            )?;

        let salt = encryption_salt;

        macro_rules! enc_opt {
            ($field:expr) => {
                if let Some(val) = $field.as_deref() {
                    Some(crypto.encrypt(val, salt)?)
                } else {
                    None
                }
            };
        }

        let stress_enc = enc_opt!(req.stress_level);
        let anxiety_enc = enc_opt!(req.anxiety_level);
        let depression_enc = enc_opt!(req.depression_level);
        let sleep_enc = enc_opt!(req.sleep_quality);
        let social_enc = enc_opt!(req.social_support);
        let coping_enc = enc_opt!(req.coping_style);
        let personality_enc = enc_opt!(req.personality_traits);
        let history_enc = enc_opt!(req.mental_health_history);
        let medications_enc = enc_opt!(req.current_medications);
        let therapy_enc = enc_opt!(req.therapy_status);
        let notes_enc = enc_opt!(req.additional_notes);
        let family_enc = enc_opt!(req.family_background);

        // Recalculate risk level using current or updated values
        let stress = req.stress_level
            .as_deref()
            .unwrap_or_else(|| {
                crypto.decrypt(&existing.stress_level_enc, salt).unwrap_or_default().leak()
            });
        let anxiety = req.anxiety_level
            .as_deref()
            .unwrap_or_else(|| {
                crypto.decrypt(&existing.anxiety_level_enc, salt).unwrap_or_default().leak()
            });
        let depression = req.depression_level
            .as_deref()
            .unwrap_or_else(|| {
                crypto.decrypt(&existing.depression_level_enc, salt).unwrap_or_default().leak()
            });
        let risk_level = compute_risk_level(stress, anxiety, depression);

        // Build dynamic update query
        sqlx
            ::query(
                r#"
            UPDATE mental_characteristics SET
                risk_level = ?,
                assessment_version = assessment_version + 1,
                family_background_enc = COALESCE(?, family_background_enc),
                stress_level_enc = COALESCE(?, stress_level_enc),
                anxiety_level_enc = COALESCE(?, anxiety_level_enc),
                depression_level_enc = COALESCE(?, depression_level_enc),
                sleep_quality_enc = COALESCE(?, sleep_quality_enc),
                social_support_enc = COALESCE(?, social_support_enc),
                coping_style_enc = COALESCE(?, coping_style_enc),
                personality_traits_enc = COALESCE(?, personality_traits_enc),
                mental_health_history_enc = COALESCE(?, mental_health_history_enc),
                current_medications_enc = COALESCE(?, current_medications_enc),
                therapy_status_enc = COALESCE(?, therapy_status_enc),
                additional_notes_enc = COALESCE(?, additional_notes_enc),
                updated_at = NOW()
            WHERE user_id = ?
        "#
            )
            .bind(&risk_level)
            .bind(family_enc.as_deref())
            .bind(stress_enc.as_deref())
            .bind(anxiety_enc.as_deref())
            .bind(depression_enc.as_deref())
            .bind(sleep_enc.as_deref())
            .bind(social_enc.as_deref())
            .bind(coping_enc.as_deref())
            .bind(personality_enc.as_deref())
            .bind(history_enc.as_deref())
            .bind(medications_enc.as_deref())
            .bind(therapy_enc.as_deref())
            .bind(notes_enc.as_deref())
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let updated: MentalCharacteristicRow = sqlx
            ::query_as("SELECT * FROM mental_characteristics WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &updated, encryption_salt)
    }

    fn decrypt_row(
        crypto: &CryptoService,
        row: &MentalCharacteristicRow,
        salt: &str
    ) -> Result<BaselineAssessmentResponse> {
        Ok(BaselineAssessmentResponse {
            id: row.id.clone(),
            user_id: row.user_id.clone(),
            risk_level: row.risk_level.clone(),
            family_background: crypto.decrypt_optional(row.family_background_enc.as_deref(), salt)?,
            stress_level: crypto.decrypt(&row.stress_level_enc, salt)?,
            anxiety_level: crypto.decrypt(&row.anxiety_level_enc, salt)?,
            depression_level: crypto.decrypt(&row.depression_level_enc, salt)?,
            sleep_quality: crypto.decrypt(&row.sleep_quality_enc, salt)?,
            social_support: crypto.decrypt(&row.social_support_enc, salt)?,
            coping_style: crypto.decrypt(&row.coping_style_enc, salt)?,
            personality_traits: crypto.decrypt(&row.personality_traits_enc, salt)?,
            mental_health_history: crypto.decrypt(&row.mental_health_history_enc, salt)?,
            current_medications: crypto.decrypt_optional(
                row.current_medications_enc.as_deref(),
                salt
            )?,
            therapy_status: crypto.decrypt(&row.therapy_status_enc, salt)?,
            additional_notes: crypto.decrypt_optional(row.additional_notes_enc.as_deref(), salt)?,
            created_at: row.created_at.to_string(),
        })
    }
}

/// Compute a simple risk level from stress/anxiety/depression categorical values.
fn compute_risk_level(stress: &str, anxiety: &str, depression: &str) -> String {
    let score = |s: &str| {
        match s.to_lowercase().as_str() {
            "low" => 1u8,
            "moderate" => 2,
            "high" => 3,
            "severe" => 4,
            _ => 1,
        }
    };
    let total = score(stress) + score(anxiety) + score(depression);
    match total {
        3..=4 => "low".into(),
        5..=7 => "moderate".into(),
        8..=10 => "high".into(),
        _ => "severe".into(),
    }
}
