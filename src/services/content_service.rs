use sqlx::MySqlPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::content::{
        ContentListItem, ContentResponse, ContentRow, CreateContentRequest, UpdateContentRequest,
    },
};

pub struct ContentService;

impl ContentService {
    /// Create a new content entry (admin only).
    pub async fn create_content(
        pool: &MySqlPool,
        author_id: &str,
        req: &CreateContentRequest,
        image_base_url: &str,
    ) -> Result<ContentResponse> {
        if req.title.trim().is_empty() {
            return Err(AppError::ValidationError("Title cannot be empty".into()));
        }
        if req.body.trim().is_empty() {
            return Err(AppError::ValidationError("Body cannot be empty".into()));
        }

        let status = req.status.as_deref().unwrap_or("draft");
        Self::validate_status(status)?;

        let id = Uuid::new_v4().to_string();
        let slug = Self::generate_slug(&req.title);

        let slug = Self::ensure_unique_slug(pool, &slug).await?;

        let published_at = if status == "published" {
            "NOW()"
        } else {
            "NULL"
        };

        let query = format!(
            r#"INSERT INTO contents (id, author_id, title, slug, body, excerpt, status, published_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, {})"#,
            published_at
        );

        sqlx::query(&query)
            .bind(&id)
            .bind(author_id)
            .bind(req.title.trim())
            .bind(&slug)
            .bind(req.body.trim())
            .bind(req.excerpt.as_deref().map(str::trim))
            .bind(status)
            .execute(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        let row: ContentRow = sqlx::query_as("SELECT * FROM contents WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(row.to_response(image_base_url))
    }

    /// List contents. Admin sees all statuses; public sees only published.
    pub async fn list_contents(
        pool: &MySqlPool,
        is_admin: bool,
        limit: i64,
        offset: i64,
        image_base_url: &str,
    ) -> Result<(Vec<ContentListItem>, i64)> {
        let (rows, total): (Vec<ContentRow>, (i64,)) = if is_admin {
            let rows: Vec<ContentRow> = sqlx::query_as(
                r#"SELECT * FROM contents ORDER BY created_at DESC LIMIT ? OFFSET ?"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(AppError::DatabaseError)?;

            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents")
                .fetch_one(pool)
                .await
                .map_err(AppError::DatabaseError)?;

            (rows, total)
        } else {
            let rows: Vec<ContentRow> = sqlx::query_as(
                r#"SELECT * FROM contents WHERE status = 'published'
                   ORDER BY published_at DESC LIMIT ? OFFSET ?"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(AppError::DatabaseError)?;

            let total: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM contents WHERE status = 'published'")
                    .fetch_one(pool)
                    .await
                    .map_err(AppError::DatabaseError)?;

            (rows, total)
        };

        let items = rows
            .iter()
            .map(|r| r.to_list_item(image_base_url))
            .collect();
        Ok((items, total.0))
    }

    /// Get a single content by ID. Admin sees any status; public sees only published.
    pub async fn get_content(
        pool: &MySqlPool,
        content_id: &str,
        is_admin: bool,
        image_base_url: &str,
    ) -> Result<ContentResponse> {
        let row: ContentRow = (if is_admin {
            sqlx::query_as("SELECT * FROM contents WHERE id = ?")
                .bind(content_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::DatabaseError)?
        } else {
            sqlx::query_as("SELECT * FROM contents WHERE id = ? AND status = 'published'")
                .bind(content_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::DatabaseError)?
        })
        .ok_or_else(|| AppError::NotFound("Content not found".into()))?;

        Ok(row.to_response(image_base_url))
    }

    /// Get a single content by slug. Admin sees any status; public sees only published.
    pub async fn get_content_by_slug(
        pool: &MySqlPool,
        slug: &str,
        is_admin: bool,
        image_base_url: &str,
    ) -> Result<ContentResponse> {
        let row: ContentRow = (if is_admin {
            sqlx::query_as("SELECT * FROM contents WHERE slug = ?")
                .bind(slug)
                .fetch_optional(pool)
                .await
                .map_err(AppError::DatabaseError)?
        } else {
            sqlx::query_as("SELECT * FROM contents WHERE slug = ? AND status = 'published'")
                .bind(slug)
                .fetch_optional(pool)
                .await
                .map_err(AppError::DatabaseError)?
        })
        .ok_or_else(|| AppError::NotFound("Content not found".into()))?;

        Ok(row.to_response(image_base_url))
    }

    /// Update a content entry (admin only).
    pub async fn update_content(
        pool: &MySqlPool,
        content_id: &str,
        req: &UpdateContentRequest,
        image_base_url: &str,
    ) -> Result<ContentResponse> {
        let existing: ContentRow = sqlx::query_as("SELECT * FROM contents WHERE id = ?")
            .bind(content_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Content not found".into()))?;

        if let Some(ref status) = req.status {
            Self::validate_status(status)?;
        }

        let title = req.title.as_deref().unwrap_or(&existing.title);
        let body = req.body.as_deref().unwrap_or(&existing.body);
        let excerpt = req.excerpt.as_deref().or(existing.excerpt.as_deref());
        let status = req.status.as_deref().unwrap_or(&existing.status);

        let slug = if req.title.is_some() {
            let new_slug = Self::generate_slug(title);
            if new_slug != existing.slug {
                Self::ensure_unique_slug_excluding(pool, &new_slug, content_id).await?
            } else {
                existing.slug.clone()
            }
        } else {
            existing.slug.clone()
        };

        let set_published = status == "published" && existing.published_at.is_none();

        if set_published {
            sqlx::query(
                r#"UPDATE contents SET title = ?, slug = ?, body = ?, excerpt = ?, status = ?,
                   published_at = NOW(), updated_at = NOW() WHERE id = ?"#,
            )
            .bind(title.trim())
            .bind(&slug)
            .bind(body.trim())
            .bind(excerpt.map(str::trim))
            .bind(status)
            .bind(content_id)
            .execute(pool)
            .await
            .map_err(AppError::DatabaseError)?;
        } else {
            sqlx::query(
                r#"UPDATE contents SET title = ?, slug = ?, body = ?, excerpt = ?, status = ?,
                   updated_at = NOW() WHERE id = ?"#,
            )
            .bind(title.trim())
            .bind(&slug)
            .bind(body.trim())
            .bind(excerpt.map(str::trim))
            .bind(status)
            .bind(content_id)
            .execute(pool)
            .await
            .map_err(AppError::DatabaseError)?;
        }

        let row: ContentRow = sqlx::query_as("SELECT * FROM contents WHERE id = ?")
            .bind(content_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(row.to_response(image_base_url))
    }

    /// Set or replace the cover image filename for a content entry.
    pub async fn set_cover_image(
        pool: &MySqlPool,
        content_id: &str,
        filename: &str,
        image_base_url: &str,
    ) -> Result<ContentResponse> {
        sqlx::query("UPDATE contents SET cover_image = ?, updated_at = NOW() WHERE id = ?")
            .bind(filename)
            .bind(content_id)
            .execute(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        let row: ContentRow = sqlx::query_as("SELECT * FROM contents WHERE id = ?")
            .bind(content_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(row.to_response(image_base_url))
    }

    /// Delete a content entry (admin only).
    pub async fn delete_content(pool: &MySqlPool, content_id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM contents WHERE id = ?")
            .bind(content_id)
            .execute(pool)
            .await
            .map_err(AppError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Content not found".into()));
        }

        Ok(())
    }

    /// Generate a URL-safe slug from a title.
    pub fn generate_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else if c == ' ' || c == '_' || c == '-' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|c| *c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Ensure slug uniqueness by appending a numeric suffix if necessary.
    async fn ensure_unique_slug(pool: &MySqlPool, base: &str) -> Result<String> {
        let mut candidate = base.to_string();
        let mut suffix = 1u32;
        loop {
            let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contents WHERE slug = ?")
                .bind(&candidate)
                .fetch_one(pool)
                .await
                .map_err(AppError::DatabaseError)?;
            if count.0 == 0 {
                return Ok(candidate);
            }
            candidate = format!("{}-{}", base, suffix);
            suffix += 1;
        }
    }

    /// Ensure slug uniqueness excluding a specific content ID (for updates).
    async fn ensure_unique_slug_excluding(
        pool: &MySqlPool,
        base: &str,
        exclude_id: &str,
    ) -> Result<String> {
        let mut candidate = base.to_string();
        let mut suffix = 1u32;
        loop {
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM contents WHERE slug = ? AND id != ?")
                    .bind(&candidate)
                    .bind(exclude_id)
                    .fetch_one(pool)
                    .await
                    .map_err(AppError::DatabaseError)?;
            if count.0 == 0 {
                return Ok(candidate);
            }
            candidate = format!("{}-{}", base, suffix);
            suffix += 1;
        }
    }

    fn validate_status(status: &str) -> Result<()> {
        match status {
            "draft" | "published" | "archived" => Ok(()),
            _ => Err(AppError::ValidationError(
                "Status must be 'draft', 'published', or 'archived'".into(),
            )),
        }
    }
}
