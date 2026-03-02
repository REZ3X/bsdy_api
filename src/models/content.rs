use chrono::NaiveDateTime;
use serde::{ Deserialize, Serialize };

// ── Content (Blog/Article) ──────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContentRow {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub excerpt: Option<String>,
    pub cover_image: Option<String>,
    pub status: String,
    pub published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct ContentResponse {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub excerpt: Option<String>,
    pub cover_image_url: Option<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Lightweight listing response (no full body).
#[derive(Debug, Serialize)]
pub struct ContentListItem {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub cover_image_url: Option<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
}

impl ContentRow {
    /// Convert to full response with optional image base URL.
    pub fn to_response(&self, image_base_url: &str) -> ContentResponse {
        ContentResponse {
            id: self.id.clone(),
            author_id: self.author_id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            body: self.body.clone(),
            excerpt: self.excerpt.clone(),
            cover_image_url: self.cover_image
                .as_ref()
                .map(|img| { format!("{}/uploads/content/{}", image_base_url, img) }),
            status: self.status.clone(),
            published_at: self.published_at.map(|d| d.to_string()),
            created_at: self.created_at.to_string(),
            updated_at: self.updated_at.to_string(),
        }
    }

    /// Convert to list item (no body) for listing endpoints.
    pub fn to_list_item(&self, image_base_url: &str) -> ContentListItem {
        ContentListItem {
            id: self.id.clone(),
            author_id: self.author_id.clone(),
            title: self.title.clone(),
            slug: self.slug.clone(),
            excerpt: self.excerpt.clone(),
            cover_image_url: self.cover_image
                .as_ref()
                .map(|img| { format!("{}/uploads/content/{}", image_base_url, img) }),
            status: self.status.clone(),
            published_at: self.published_at.map(|d| d.to_string()),
            created_at: self.created_at.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub title: String,
    pub body: String,
    pub excerpt: Option<String>,
    pub status: Option<String>, // draft | published | archived (default: draft)
}

#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub excerpt: Option<String>,
    pub status: Option<String>,
}
