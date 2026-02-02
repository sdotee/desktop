use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub original_url: String,
    pub short_url: String,
    pub domain: String,
    pub slug: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl LinkEntry {
    pub fn new(
        original_url: String,
        short_url: String,
        domain: String,
        slug: String,
        title: Option<String>,
    ) -> Self {
        Self {
            original_url,
            short_url,
            domain,
            slug,
            title,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntry {
    pub url: String,
    #[serde(default)]
    pub page_url: Option<String>,
    pub domain: String,
    pub slug: String,
    pub title: Option<String>,
    pub syntax: Option<String>,
    pub content_preview: String,
    pub created_at: DateTime<Utc>,
}

impl TextEntry {
    pub fn new(
        url: String,
        page_url: Option<String>,
        domain: String,
        slug: String,
        title: Option<String>,
        syntax: Option<String>,
        content_preview: String,
    ) -> Self {
        Self {
            url,
            page_url,
            domain,
            slug,
            title,
            syntax,
            content_preview,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub url: String,
    #[serde(default)]
    pub page_url: Option<String>,
    pub domain: String,
    pub slug: String,
    pub filename: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FileEntry {
    pub fn new(
        url: String,
        page_url: Option<String>,
        domain: String,
        slug: String,
        filename: String,
        size: u64,
        mime_type: Option<String>,
    ) -> Self {
        Self {
            url,
            page_url,
            domain,
            slug,
            filename,
            size,
            mime_type,
            created_at: Utc::now(),
        }
    }
}
