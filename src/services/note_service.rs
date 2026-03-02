use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    crypto::CryptoService,
    error::{ AppError, Result },
    models::note::{ CreateNoteRequest, NoteResponse, NoteRow, UpdateNoteRequest },
};

pub struct NoteService;

impl NoteService {
    /// Create a new note (coping toolkit entry).
    pub async fn create_note(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        req: &CreateNoteRequest
    ) -> Result<NoteResponse> {
        if req.title.trim().is_empty() {
            return Err(AppError::ValidationError("Title cannot be empty".into()));
        }
        if req.content.trim().is_empty() {
            return Err(AppError::ValidationError("Content cannot be empty".into()));
        }

        let id = Uuid::new_v4().to_string();
        let title_enc = crypto.encrypt(&req.title, encryption_salt)?;
        let content_enc = crypto.encrypt(&req.content, encryption_salt)?;
        let is_pinned = req.is_pinned.unwrap_or(false);

        sqlx
            ::query(
                r#"INSERT INTO notes (id, user_id, title_enc, content_enc, label, is_pinned)
               VALUES (?, ?, ?, ?, ?, ?)"#
            )
            .bind(&id)
            .bind(user_id)
            .bind(&title_enc)
            .bind(&content_enc)
            .bind(req.label.as_deref())
            .bind(is_pinned)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: NoteRow = sqlx
            ::query_as("SELECT * FROM notes WHERE id = ?")
            .bind(&id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Get all notes for a user with optional label filter.
    pub async fn get_notes(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        encryption_salt: &str,
        label: Option<&str>,
        limit: i64
    ) -> Result<Vec<NoteResponse>> {
        let rows: Vec<NoteRow> = if let Some(label) = label {
            sqlx
                ::query_as(
                    r#"SELECT * FROM notes
                   WHERE user_id = ? AND label = ?
                   ORDER BY is_pinned DESC, updated_at DESC
                   LIMIT ?"#
                )
                .bind(user_id)
                .bind(label)
                .bind(limit)
                .fetch_all(pool).await
                .map_err(AppError::DatabaseError)?
        } else {
            sqlx
                ::query_as(
                    r#"SELECT * FROM notes
                   WHERE user_id = ?
                   ORDER BY is_pinned DESC, updated_at DESC
                   LIMIT ?"#
                )
                .bind(user_id)
                .bind(limit)
                .fetch_all(pool).await
                .map_err(AppError::DatabaseError)?
        };

        rows.iter()
            .map(|r| Self::decrypt_row(crypto, r, encryption_salt))
            .collect()
    }

    /// Get a single note by ID.
    pub async fn get_note(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        note_id: &str,
        encryption_salt: &str
    ) -> Result<NoteResponse> {
        let row: NoteRow = sqlx
            ::query_as("SELECT * FROM notes WHERE id = ? AND user_id = ?")
            .bind(note_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Note not found".into()))?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Update a note.
    pub async fn update_note(
        pool: &MySqlPool,
        crypto: &CryptoService,
        user_id: &str,
        note_id: &str,
        encryption_salt: &str,
        req: &UpdateNoteRequest
    ) -> Result<NoteResponse> {
        // Check existence
        let _existing: NoteRow = sqlx
            ::query_as("SELECT * FROM notes WHERE id = ? AND user_id = ?")
            .bind(note_id)
            .bind(user_id)
            .fetch_optional(pool).await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Note not found".into()))?;

        let title_enc = if let Some(ref title) = req.title {
            if title.trim().is_empty() {
                return Err(AppError::ValidationError("Title cannot be empty".into()));
            }
            Some(crypto.encrypt(title, encryption_salt)?)
        } else {
            None
        };

        let content_enc = if let Some(ref content) = req.content {
            if content.trim().is_empty() {
                return Err(AppError::ValidationError("Content cannot be empty".into()));
            }
            Some(crypto.encrypt(content, encryption_salt)?)
        } else {
            None
        };

        sqlx
            ::query(
                r#"UPDATE notes SET
                title_enc = COALESCE(?, title_enc),
                content_enc = COALESCE(?, content_enc),
                label = COALESCE(?, label),
                is_pinned = COALESCE(?, is_pinned),
                updated_at = NOW()
               WHERE id = ? AND user_id = ?"#
            )
            .bind(title_enc.as_deref())
            .bind(content_enc.as_deref())
            .bind(req.label.as_deref())
            .bind(req.is_pinned)
            .bind(note_id)
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        let row: NoteRow = sqlx
            ::query_as("SELECT * FROM notes WHERE id = ?")
            .bind(note_id)
            .fetch_one(pool).await
            .map_err(AppError::DatabaseError)?;

        Self::decrypt_row(crypto, &row, encryption_salt)
    }

    /// Delete a note.
    pub async fn delete_note(pool: &MySqlPool, user_id: &str, note_id: &str) -> Result<()> {
        let result = sqlx
            ::query("DELETE FROM notes WHERE id = ? AND user_id = ?")
            .bind(note_id)
            .bind(user_id)
            .execute(pool).await
            .map_err(AppError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Note not found".into()));
        }

        Ok(())
    }

    /// Get distinct labels used by a user.
    pub async fn get_labels(pool: &MySqlPool, user_id: &str) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx
            ::query_as(
                "SELECT DISTINCT label FROM notes WHERE user_id = ? AND label IS NOT NULL ORDER BY label"
            )
            .bind(user_id)
            .fetch_all(pool).await
            .map_err(AppError::DatabaseError)?;

        Ok(
            rows
                .into_iter()
                .map(|(l,)| l)
                .collect()
        )
    }

    fn decrypt_row(crypto: &CryptoService, row: &NoteRow, salt: &str) -> Result<NoteResponse> {
        Ok(NoteResponse {
            id: row.id.clone(),
            title: crypto.decrypt(&row.title_enc, salt)?,
            content: crypto.decrypt(&row.content_enc, salt)?,
            label: row.label.clone(),
            is_pinned: row.is_pinned,
            created_at: row.created_at.to_string(),
            updated_at: row.updated_at.to_string(),
        })
    }
}
