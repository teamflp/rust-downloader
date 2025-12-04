use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    #[serde(rename = "type")]
    pub download_type: DownloadType,
    pub format: Option<String>,
    pub quality: Option<String>,
    pub custom_filename: Option<String>,
    pub cookies_browser: Option<String>,
    pub download_playlist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    Video,
    Audio,
    Instrumental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Processing,
    Completed,
    Failed,
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

#[derive(Debug, Serialize)]
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
