use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DownloadRequest {
    pub url: String,
    #[serde(rename = "type")]
    pub download_type: DownloadType,
    pub format: Option<String>,
    pub resolution: Option<String>, // e.g., "1080p", "720p", "480p", "best"
    pub audio_quality: Option<String>, // e.g., "128k", "192k", "256k", "320k", "best"
    pub custom_filename: Option<String>,
    pub cookies_browser: Option<String>,
    pub download_playlist: Option<bool>,
    pub download_subtitles: Option<bool>,
    pub subtitle_language: Option<String>, // e.g., "fr", "en", "auto"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    Video,
    Audio,
    Instrumental,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DownloadResponse {
    pub id: String,
    pub status: DownloadStatus,
    pub progress: f32,
    pub message: String,
    pub url: String,
    pub download_type: DownloadType,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub is_playlist: bool,
    pub total_items: Option<i32>,
    pub completed_items: Option<i32>,
    // Metadata fields
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub duration: Option<u64>, // in seconds
    pub author: Option<String>,
    pub file_size: Option<u64>, // in bytes
    // Retry fields
    pub retry_count: Option<u32>,
    pub max_retries: Option<u32>,
    // User-editable metadata
    pub notes: Option<String>,
    // Tags
    pub tags: Option<Vec<Tag>>,
    // Original file path before conversion
    pub original_file_path: Option<String>,
    // Favorite status
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Processing,
    Completed,
    Failed,
    Converting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub category: Option<String>,
}

impl DownloadResponse {
    pub fn new(url: String, download_type: DownloadType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            status: DownloadStatus::Pending,
            progress: 0.0,
            message: "Download queued".to_string(),
            url,
            download_type,
            created_at: Utc::now(),
            completed_at: None,
            file_path: None,
            is_playlist: false,
            total_items: None,
            completed_items: None,
            title: None,
            thumbnail: None,
            duration: None,
            author: None,
            file_size: None,
            retry_count: Some(0),
            max_retries: Some(3),
            notes: None,
            tags: None,
            original_file_path: None,
            is_favorite: false,
        }
    }

    #[allow(dead_code)]
    pub fn update_progress(&mut self, progress: f32, message: String) {
        self.progress = progress;
        self.message = message;
        if progress >= 100.0 {
            self.status = DownloadStatus::Completed;
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn set_status(&mut self, status: DownloadStatus, message: String) {
        self.status = status;
        self.message = message;
        if status == DownloadStatus::Completed {
            self.completed_at = Some(Utc::now());
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(as = utoipa::openapi::Object)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMetadataRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConvertFileRequest {
    pub format: String, // e.g., "mp4", "mp3", "webm", etc.
    pub keep_original: Option<bool>, // Whether to keep the original file
}

// Statistics models
#[derive(Debug, Serialize)]
pub struct DownloadTrendPoint {
    pub date: String, // ISO date string (YYYY-MM-DD)
    pub count: i64,
    pub total_size: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TypeDistribution {
    pub download_type: String,
    pub count: i64,
    pub total_size: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusDistribution {
    pub status: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpaceEvolutionPoint {
    pub date: String, // ISO date string (YYYY-MM-DD)
    pub total_size: i64,
    pub file_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatisticsResponse {
    pub trends: Vec<DownloadTrendPoint>, // Downloads by day
    pub type_distribution: Vec<TypeDistribution>,
    pub status_distribution: Vec<StatusDistribution>,
    pub space_evolution: Vec<SpaceEvolutionPoint>,
    pub success_rate: f64, // Percentage of successful downloads
    pub total_downloads: i64,
    pub total_size: i64,
    pub average_file_size: i64,
}

// Webhooks
#[derive(Debug, Serialize, Clone, Deserialize, ToSchema)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub events: Vec<String>, // e.g., ["download.completed", "download.failed"]
    pub secret: Option<String>, // Optional secret for HMAC signing
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_triggered_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct WebhookEvent {
    pub event_type: String, // e.g., "download.completed"
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}


