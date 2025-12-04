use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::models::{DownloadResponse, DownloadStatus, DownloadType};
use anyhow::Result;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                download_type TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL NOT NULL DEFAULT 0.0,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                file_path TEXT,
                is_playlist BOOLEAN NOT NULL DEFAULT 0,
                total_items INTEGER,
                completed_items INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn insert_download(&self, download: &DownloadResponse) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO downloads (id, url, download_type, status, progress, message, created_at, completed_at, file_path, is_playlist, total_items, completed_items)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&download.id)
        .bind(&download.url)
        .bind(download_type_to_string(&download.download_type))
        .bind(status_to_string(&download.status))
        .bind(download.progress)
        .bind(&download.message)
        .bind(download.created_at.to_rfc3339())
        .bind(download.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(&download.file_path)
        .bind(download.is_playlist)
        .bind(download.total_items)
        .bind(download.completed_items)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_download(&self, download: &DownloadResponse) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE downloads
            SET status = ?, progress = ?, message = ?, completed_at = ?, file_path = ?, is_playlist = ?, total_items = ?, completed_items = ?
            WHERE id = ?
            "#,
        )
        .bind(status_to_string(&download.status))
        .bind(download.progress)
        .bind(&download.message)
        .bind(download.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(&download.file_path)
        .bind(download.is_playlist)
        .bind(download.total_items)
        .bind(download.completed_items)
        .bind(&download.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_download(&self, id: &str) -> Result<Option<DownloadResponse>> {
        let row = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_all_downloads(&self) -> Result<Vec<DownloadResponse>> {
        let rows = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_download(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM downloads WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    #[allow(dead_code)]
    pub async fn get_active_downloads(&self) -> Result<Vec<DownloadResponse>> {
        let rows = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads WHERE status IN ('pending', 'downloading', 'processing') ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct DownloadRow {
    id: String,
    url: String,
    download_type: String,
    status: String,
    progress: f32,
    message: String,
    created_at: String,
    completed_at: Option<String>,
    file_path: Option<String>,
    is_playlist: bool,
    total_items: Option<i32>,
    completed_items: Option<i32>,
}

impl From<DownloadRow> for DownloadResponse {
    fn from(row: DownloadRow) -> Self {
        DownloadResponse {
            id: row.id,
            url: row.url,
            download_type: string_to_download_type(&row.download_type),
            status: string_to_status(&row.status),
            progress: row.progress,
            message: row.message,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
            completed_at: row.completed_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            file_path: row.file_path,
            is_playlist: row.is_playlist,
            total_items: row.total_items,
            completed_items: row.completed_items,
        }
    }
}

fn download_type_to_string(dt: &DownloadType) -> String {
    match dt {
        DownloadType::Video => "video".to_string(),
        DownloadType::Audio => "audio".to_string(),
        DownloadType::Instrumental => "instrumental".to_string(),
    }
}

fn string_to_download_type(s: &str) -> DownloadType {
    match s {
        "video" => DownloadType::Video,
        "audio" => DownloadType::Audio,
        "instrumental" => DownloadType::Instrumental,
        _ => DownloadType::Video,
    }
}

fn status_to_string(status: &DownloadStatus) -> String {
    match status {
        DownloadStatus::Pending => "pending".to_string(),
        DownloadStatus::Downloading => "downloading".to_string(),
        DownloadStatus::Processing => "processing".to_string(),
        DownloadStatus::Completed => "completed".to_string(),
        DownloadStatus::Failed => "failed".to_string(),
    }
}

fn string_to_status(s: &str) -> DownloadStatus {
    match s {
        "pending" => DownloadStatus::Pending,
        "downloading" => DownloadStatus::Downloading,
        "processing" => DownloadStatus::Processing,
        "completed" => DownloadStatus::Completed,
        "failed" => DownloadStatus::Failed,
        _ => DownloadStatus::Pending,
    }
}
