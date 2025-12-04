use std::sync::Arc;
use crate::models::DownloadResponse;
use crate::db::Database;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Database>,
}

impl AppState {
    pub fn new_with_db(db: Database) -> Self {
        Self {
            db: Arc::new(db),
        }
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
}
