use std::sync::Arc;
use crate::models::{DownloadResponse, Tag, CreateTagRequest};
use crate::db::Database;
use crate::cache::VideoInfoCache;
use tracing::warn;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Database>,
    video_cache: Arc<VideoInfoCache>,
}

impl AppState {
    pub fn new_with_db(db: Database) -> Self {
        use std::time::Duration;
        Self {
            db: Arc::new(db),
            video_cache: Arc::new(VideoInfoCache::new(Duration::from_secs(3600))), // 1 hour TTL
        }
    }

    pub fn get_video_cache(&self) -> Arc<VideoInfoCache> {
        self.video_cache.clone()
    }

    pub async fn add_download(&self, download: DownloadResponse) {
        if let Err(e) = self.db.insert_download(&download).await {
            tracing::error!("Failed to insert download: {}", e);
        }
    }

    pub async fn get_download(&self, id: &str) -> Option<DownloadResponse> {
        self.db.get_download(id).await.ok().flatten()
    }

    pub async fn update_download(&self, id: &str, download: DownloadResponse) {
        if let Err(e) = self.db.update_download(&download).await {
            tracing::error!("Failed to update download {}: {}", id, e);
        }
    }

    pub async fn get_all_downloads(&self) -> Vec<DownloadResponse> {
        self.db.get_all_downloads().await.unwrap_or_default()
    }

    pub async fn get_downloads_paginated(&self, page: u32, per_page: u32) -> (Vec<DownloadResponse>, u64) {
        self.db.get_downloads_paginated(page, per_page).await.unwrap_or((Vec::new(), 0))
    }

    pub async fn remove_download(&self, id: &str) -> Option<DownloadResponse> {
        // Get the download first
        let download = self.get_download(id).await;
        
        // Delete from database
        if let Err(e) = self.db.delete_download(id).await {
            tracing::error!("Failed to delete download {}: {}", id, e);
            return None;
        }
        
        download
    }

    pub async fn update_metadata(&self, id: &str, title: Option<&str>, author: Option<&str>, notes: Option<&str>) -> anyhow::Result<()> {
        self.db.update_metadata(id, title, author, notes).await
    }

    // Tags methods
    pub async fn create_tag(&self, request: CreateTagRequest) -> anyhow::Result<Tag> {
        use uuid::Uuid;
        use chrono::Utc;
        
        let tag = Tag {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            color: request.color,
            category: request.category,
            created_at: Utc::now(),
        };
        
        self.db.create_tag(&tag).await?;
        Ok(tag)
    }

    pub async fn get_tag(&self, id: &str) -> Option<Tag> {
        self.db.get_tag(id).await.ok().flatten()
    }

    pub async fn get_tag_by_name(&self, name: &str) -> Option<Tag> {
        self.db.get_tag_by_name(name).await.ok().flatten()
    }

    pub async fn get_all_tags(&self) -> Vec<Tag> {
        self.db.get_all_tags().await.unwrap_or_default()
    }

    pub async fn update_tag(&self, tag: Tag) -> anyhow::Result<()> {
        self.db.update_tag(&tag).await
    }

    pub async fn delete_tag(&self, id: &str) -> bool {
        self.db.delete_tag(id).await.unwrap_or(false)
    }

    pub async fn get_download_tags(&self, download_id: &str) -> Vec<Tag> {
        self.db.get_download_tags(download_id).await.unwrap_or_default()
    }

    pub async fn add_tag_to_download(&self, download_id: &str, tag_id: &str) -> anyhow::Result<()> {
        self.db.add_tag_to_download(download_id, tag_id).await
    }

    pub async fn remove_tag_from_download(&self, download_id: &str, tag_id: &str) -> anyhow::Result<()> {
        self.db.remove_tag_from_download(download_id, tag_id).await
    }

    pub async fn set_download_tags(&self, download_id: &str, tag_ids: &[String]) -> anyhow::Result<()> {
        self.db.set_download_tags(download_id, tag_ids).await
    }

    pub async fn convert_download(&self, id: &str, format: &str, keep_original: bool) -> anyhow::Result<()> {
        use crate::converter::{convert_file, ConversionFormat};
        use rust_media_downloader_shared::config;
        use std::path::PathBuf;
        use std::fs;

        // Get the download
        let mut download = match self.get_download(id).await {
            Some(d) => d,
            None => anyhow::bail!("Download not found"),
        };

        // Check if download is completed
        if download.status != crate::models::DownloadStatus::Completed {
            anyhow::bail!("Download must be completed before conversion");
        }

        // Get file path
        let file_path_str = match &download.file_path {
            Some(path) => path,
            None => anyhow::bail!("File path not found"),
        };

        // Load config to get download directory
        let config = config::load_config();
        let download_dir = PathBuf::from(&config.download_directory);
        
        // Construct full path (same logic as serve_file)
        let input_path = if PathBuf::from(file_path_str).is_absolute() {
            PathBuf::from(file_path_str)
        } else {
            download_dir.join(file_path_str)
        };

        if !input_path.exists() {
            anyhow::bail!("Input file does not exist at: {}. The file may have been moved or deleted.", input_path.display());
        }

        // Parse format
        let conversion_format = ConversionFormat::from_str(format)
            .ok_or_else(|| {
                tracing::error!("Unsupported conversion format: {}", format);
                anyhow::anyhow!("Format non supporté: {}. Formats disponibles: mp4, webm, mkv, mp3, wav, flac, m4a, aac", format)
            })?;

        // Update status to converting
        download.status = crate::models::DownloadStatus::Converting;
        download.progress = 0.0;
        download.message = format!("Conversion vers {}...", format);
        self.update_download(id, download.clone()).await;

        // Store original file path if not already stored
        if download.original_file_path.is_none() {
            download.original_file_path = download.file_path.clone();
        }

        // Convert file with progress callback
        let state_clone = self.clone();
        let id_clone = id.to_string();
        
        let progress_callback = Box::new(move |progress: f32| {
            let state = state_clone.clone();
            let id = id_clone.clone();
            tokio::spawn(async move {
                if let Some(mut dl) = state.get_download(&id).await {
                    dl.progress = progress;
                    dl.message = format!("Conversion {}%...", progress as u32);
                    let _ = state.update_download(&id, dl).await;
                }
            });
        });

        match convert_file(&input_path, conversion_format, Some(progress_callback)).await {
            Ok(output_path) => {
                // Update download with new file path
                download.file_path = Some(output_path.to_string_lossy().to_string());
                download.status = crate::models::DownloadStatus::Completed;
                download.progress = 100.0;
                download.message = format!("Conversion vers {} terminée", format);
                
                // Update file size
                if let Ok(metadata) = fs::metadata(&output_path) {
                    download.file_size = Some(metadata.len());
                }

                // Delete original file if requested
                if !keep_original && download.original_file_path.as_ref().map(|p| PathBuf::from(p)) != Some(input_path.clone()) {
                    if let Err(e) = fs::remove_file(&input_path) {
                        warn!("Failed to delete original file: {}", e);
                    }
                }

                self.update_download(id, download).await;
                Ok(())
            }
            Err(e) => {
                // Log error for debugging
                tracing::error!("Conversion failed for download {}: {}", id, e);
                // Mark as failed
                download.status = crate::models::DownloadStatus::Failed;
                download.progress = 0.0;
                download.message = format!("Échec de la conversion: {}", e);
                self.update_download(id, download).await;
                Err(e)
            }
        }
    }

    pub async fn toggle_favorite(&self, id: &str) -> anyhow::Result<()> {
        let mut download = match self.get_download(id).await {
            Some(d) => d,
            None => anyhow::bail!("Download not found"),
        };

        download.is_favorite = !download.is_favorite;
        self.update_download(id, download).await;
        Ok(())
    }

    pub async fn get_statistics(&self) -> anyhow::Result<crate::models::StatisticsResponse> {
        self.db.get_statistics().await
    }

    // Webhooks methods
    pub async fn create_webhook(&self, url: &str, events: &[String], secret: Option<&str>) -> anyhow::Result<crate::models::Webhook> {
        self.db.create_webhook(url, events, secret).await
    }

    pub async fn get_all_webhooks(&self) -> anyhow::Result<Vec<crate::models::Webhook>> {
        self.db.get_all_webhooks().await
    }

    pub async fn get_active_webhooks_for_event(&self, event_type: &str) -> anyhow::Result<Vec<crate::models::Webhook>> {
        self.db.get_active_webhooks_for_event(event_type).await
    }

    pub async fn update_webhook_last_triggered(&self, id: &str) -> anyhow::Result<()> {
        self.db.update_webhook_last_triggered(id).await
    }

    pub async fn delete_webhook(&self, id: &str) -> anyhow::Result<bool> {
        self.db.delete_webhook(id).await
    }
}
