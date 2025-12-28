use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::models::{DownloadResponse, DownloadStatus, DownloadType, Tag, DownloadTrendPoint, TypeDistribution, StatusDistribution, SpaceEvolutionPoint, StatisticsResponse};
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
                completed_items INTEGER DEFAULT 0,
                title TEXT,
                thumbnail TEXT,
                duration INTEGER,
                author TEXT,
                file_size INTEGER,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 3
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Migrate existing tables - add new columns if they don't exist
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN title TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN thumbnail TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN duration INTEGER").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN author TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN file_size INTEGER").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN retry_count INTEGER DEFAULT 0").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN max_retries INTEGER DEFAULT 3").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN notes TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN original_file_path TEXT").execute(&pool).await;
        let _ = sqlx::query("ALTER TABLE downloads ADD COLUMN is_favorite BOOLEAN DEFAULT 0").execute(&pool).await;
        // Initialize existing rows to 0 (false) for is_favorite
        let _ = sqlx::query("UPDATE downloads SET is_favorite = 0 WHERE is_favorite IS NULL").execute(&pool).await;

        // Create tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                category TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create download_tags junction table (many-to-many)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS download_tags (
                download_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (download_id, tag_id),
                FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        let database = Self { pool };

        // Initialize default/predefined tags if they don't exist
        database.initialize_default_tags().await?;

        // Create webhooks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webhooks (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                events TEXT NOT NULL,
                secret TEXT,
                is_active BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                last_triggered_at TEXT
            )
            "#
        )
        .execute(&database.pool)
        .await?;

        Ok(database)
    }

    pub async fn insert_download(&self, download: &DownloadResponse) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO downloads (id, url, download_type, status, progress, message, created_at, completed_at, file_path, is_playlist, total_items, completed_items, title, thumbnail, duration, author, file_size, retry_count, max_retries, notes, original_file_path, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(&download.title)
        .bind(&download.thumbnail)
        .bind(download.duration.map(|d| d as i64))
        .bind(&download.author)
        .bind(download.file_size.map(|s| s as i64))
        .bind(download.retry_count.map(|r| r as i64))
        .bind(download.max_retries.map(|r| r as i64))
        .bind(&download.notes)
        .bind(&download.original_file_path)
        .bind(download.is_favorite)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_download(&self, download: &DownloadResponse) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE downloads
            SET status = ?, progress = ?, message = ?, completed_at = ?, file_path = ?, is_playlist = ?, total_items = ?, completed_items = ?, title = ?, thumbnail = ?, duration = ?, author = ?, file_size = ?, retry_count = ?, max_retries = ?, notes = ?, original_file_path = ?, is_favorite = ?
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
        .bind(&download.title)
        .bind(&download.thumbnail)
        .bind(download.duration.map(|d| d as i64))
        .bind(&download.author)
        .bind(download.file_size.map(|s| s as i64))
        .bind(download.retry_count.map(|r| r as i64))
        .bind(download.max_retries.map(|r| r as i64))
        .bind(&download.notes)
        .bind(&download.original_file_path)
        .bind(download.is_favorite)
        .bind(&download.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_metadata(&self, id: &str, title: Option<&str>, author: Option<&str>, notes: Option<&str>) -> Result<()> {
        // Build dynamic query based on what fields are provided
        let mut query_parts = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(t) = title {
            query_parts.push("title = ?");
            values.push(t.to_string());
        }
        if let Some(a) = author {
            query_parts.push("author = ?");
            values.push(a.to_string());
        }
        if let Some(n) = notes {
            query_parts.push("notes = ?");
            values.push(n.to_string());
        }

        if query_parts.is_empty() {
            return Ok(()); // Nothing to update
        }

        let query_str = format!(
            "UPDATE downloads SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        let mut query = sqlx::query(&query_str);
        for value in values {
            query = query.bind(value);
        }
        query = query.bind(id);

        query.execute(&self.pool).await?;

        Ok(())
    }

    pub async fn get_download(&self, id: &str) -> Result<Option<DownloadResponse>> {
        let row = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let mut download: DownloadResponse = row.into();
                // Load tags
                let tags = self.get_download_tags(id).await.unwrap_or_default();
                download.tags = Some(tags);
                Ok(Some(download))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_downloads(&self) -> Result<Vec<DownloadResponse>> {
        let rows = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut downloads: Vec<DownloadResponse> = rows.into_iter().map(|r| r.into()).collect();
        
        // Load tags for each download
        for download in &mut downloads {
            if let Ok(tags) = self.get_download_tags(&download.id).await {
                download.tags = Some(tags);
            }
        }

        Ok(downloads)
    }

    pub async fn get_downloads_paginated(&self, page: u32, per_page: u32) -> Result<(Vec<DownloadResponse>, u64)> {
        let offset = (page - 1) * per_page;
        
        // Get total count
        let total_row = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM downloads")
            .fetch_one(&self.pool)
            .await?;
        let total = total_row as u64;

        // Get paginated results
        let rows = sqlx::query_as::<_, DownloadRow>(
            "SELECT * FROM downloads ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut downloads: Vec<DownloadResponse> = rows.into_iter().map(|r| r.into()).collect();
        
        // Load tags for each download
        for download in &mut downloads {
            if let Ok(tags) = self.get_download_tags(&download.id).await {
                download.tags = Some(tags);
            }
        }
        
        Ok((downloads, total))
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
    title: Option<String>,
    thumbnail: Option<String>,
    duration: Option<i64>,
    author: Option<String>,
    file_size: Option<i64>,
    retry_count: Option<i64>,
    max_retries: Option<i64>,
    notes: Option<String>,
    original_file_path: Option<String>,
    is_favorite: Option<bool>,
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
            title: row.title,
            thumbnail: row.thumbnail,
            duration: row.duration.map(|d| d as u64),
            author: row.author,
            file_size: row.file_size.map(|s| s as u64),
            retry_count: row.retry_count.map(|r| r as u32),
            max_retries: row.max_retries.map(|r| r as u32),
            notes: row.notes,
            tags: None, // Will be populated separately
            original_file_path: row.original_file_path,
            is_favorite: row.is_favorite.unwrap_or(false),
        }
    }
}

// Tags CRUD operations
impl Database {
    /// Initialize default/predefined tags if they don't exist
    pub async fn initialize_default_tags(&self) -> Result<()> {
        use uuid::Uuid;
        use chrono::Utc;

        // Default tags with categories and colors
        let default_tags = vec![
            ("Musique", "Musique", "#8b5cf6"),
            ("Films", "Films", "#ef4444"),
            ("Éducatif", "Éducatif", "#06b6d4"),
            ("Divertissement", "Divertissement", "#f59e0b"),
            ("Podcast", "Podcast", "#ec4899"),
            ("Documentaire", "Documentaire", "#10b981"),
            ("Tutoriel", "Tutoriel", "#3b82f6"),
            ("YouTube", "Plateforme", "#ff0000"),
            ("Vimeo", "Plateforme", "#1ab7ea"),
            ("TikTok", "Plateforme", "#000000"),
            ("SoundCloud", "Plateforme", "#ff7700"),
        ];

        for (name, category, color) in default_tags {
            // Check if tag already exists
            let existing = self.get_tag_by_name(name).await?;
            if existing.is_none() {
                let tag = Tag {
                    id: Uuid::new_v4().to_string(),
                    name: name.to_string(),
                    color: Some(color.to_string()),
                    category: Some(category.to_string()),
                    created_at: Utc::now(),
                };
                self.create_tag(&tag).await?;
            }
        }

        Ok(())
    }

    pub async fn create_tag(&self, tag: &Tag) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tags (id, name, color, category, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&tag.id)
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.category)
        .bind(tag.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_tag(&self, id: &str) -> Result<Option<Tag>> {
        let row = sqlx::query_as::<_, TagRow>(
            "SELECT * FROM tags WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let row = sqlx::query_as::<_, TagRow>(
            "SELECT * FROM tags WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let rows = sqlx::query_as::<_, TagRow>(
            "SELECT * FROM tags ORDER BY name ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_tag(&self, tag: &Tag) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tags
            SET name = ?, color = ?, category = ?
            WHERE id = ?
            "#,
        )
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(&tag.category)
        .bind(&tag.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_tag(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Download-Tag associations
    pub async fn get_download_tags(&self, download_id: &str) -> Result<Vec<Tag>> {
        let rows = sqlx::query_as::<_, TagRow>(
            r#"
            SELECT t.* FROM tags t
            INNER JOIN download_tags dt ON t.id = dt.tag_id
            WHERE dt.download_id = ?
            ORDER BY t.name ASC
            "#
        )
        .bind(download_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn add_tag_to_download(&self, download_id: &str, tag_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO download_tags (download_id, tag_id) VALUES (?, ?)"
        )
        .bind(download_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_tag_from_download(&self, download_id: &str, tag_id: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM download_tags WHERE download_id = ? AND tag_id = ?"
        )
        .bind(download_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_download_tags(&self, download_id: &str, tag_ids: &[String]) -> Result<()> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Remove all existing tags
        sqlx::query("DELETE FROM download_tags WHERE download_id = ?")
            .bind(download_id)
            .execute(&mut *tx)
            .await?;

        // Add new tags
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO download_tags (download_id, tag_id) VALUES (?, ?)")
                .bind(download_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // Statistics functions
    pub async fn get_download_trends(&self, days: i64) -> Result<Vec<DownloadTrendPoint>> {
        let days_str = days.to_string();
        let query = format!(
            r#"
            SELECT 
                strftime('%Y-%m-%d', created_at) as date,
                COUNT(*) as count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM downloads
            WHERE created_at >= datetime('now', '-{} days')
            GROUP BY strftime('%Y-%m-%d', created_at)
            ORDER BY date ASC
            "#,
            days_str
        );
        let rows = sqlx::query_as::<_, (String, i64, i64)>(&query)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(date, count, total_size)| DownloadTrendPoint {
            date,
            count,
            total_size,
        }).collect())
    }

    pub async fn get_type_distribution(&self) -> Result<Vec<TypeDistribution>> {
        // Get total count first
        let total_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM downloads")
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query_as::<_, (String, i64, i64)>(
            r#"
            SELECT 
                download_type,
                COUNT(*) as count,
                COALESCE(SUM(file_size), 0) as total_size
            FROM downloads
            GROUP BY download_type
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut distributions: Vec<TypeDistribution> = rows.into_iter().map(|(download_type, count, total_size)| {
            let percentage = if total_count > 0 {
                (count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };
            TypeDistribution {
                download_type,
                count,
                total_size,
                percentage,
            }
        }).collect();

        // Sort by count descending
        distributions.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(distributions)
    }

    pub async fn get_status_distribution(&self) -> Result<Vec<StatusDistribution>> {
        // Get total count first
        let total_row = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM downloads")
            .fetch_one(&self.pool)
            .await?;
        let total_count = total_row;

        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT 
                status,
                COUNT(*) as count
            FROM downloads
            GROUP BY status
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut distributions: Vec<StatusDistribution> = rows.into_iter().map(|(status, count)| {
            let percentage = if total_count > 0 {
                (count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };
            StatusDistribution {
                status,
                count,
                percentage,
            }
        }).collect();

        // Sort by count descending
        distributions.sort_by(|a, b| b.count.cmp(&a.count));

        Ok(distributions)
    }

    pub async fn get_space_evolution(&self, days: i64) -> Result<Vec<SpaceEvolutionPoint>> {
        let days_str = days.to_string();
        let query = format!(
            r#"
            SELECT 
                strftime('%Y-%m-%d', completed_at) as date,
                COALESCE(SUM(file_size), 0) as total_size,
                COUNT(*) as file_count
            FROM downloads
            WHERE completed_at IS NOT NULL 
                AND completed_at >= datetime('now', '-{} days')
                AND status = 'completed'
            GROUP BY strftime('%Y-%m-%d', completed_at)
            ORDER BY date ASC
            "#,
            days_str
        );
        let rows = sqlx::query_as::<_, (String, i64, i64)>(&query)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(date, total_size, file_count)| SpaceEvolutionPoint {
            date,
            total_size,
            file_count,
        }).collect())
    }

    pub async fn get_statistics(&self) -> Result<StatisticsResponse> {
        // Get trends for last 30 days
        let trends = self.get_download_trends(30).await?;
        
        // Get type distribution
        let type_distribution = self.get_type_distribution().await?;
        
        // Get status distribution
        let status_distribution = self.get_status_distribution().await?;
        
        // Get space evolution for last 30 days
        let space_evolution = self.get_space_evolution(30).await?;
        
        // Calculate success rate
        let success_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM downloads WHERE status = 'completed'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let total_downloads = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM downloads"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let success_rate = if total_downloads > 0 {
            (success_count as f64 / total_downloads as f64) * 100.0
        } else {
            0.0
        };
        
        // Get total size
        let total_size = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(SUM(file_size), 0) FROM downloads WHERE status = 'completed'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        // Calculate average file size
        let completed_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM downloads WHERE status = 'completed' AND file_size IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let average_file_size = if completed_count > 0 {
            total_size / completed_count
        } else {
            0
        };
        
        Ok(StatisticsResponse {
            trends,
            type_distribution,
            status_distribution,
            space_evolution,
            success_rate,
            total_downloads,
            total_size,
            average_file_size,
        })
    }

    // Webhooks methods
    pub async fn create_webhook(&self, url: &str, events: &[String], secret: Option<&str>) -> Result<crate::models::Webhook> {
        use uuid::Uuid;
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now();
        let events_json = serde_json::to_string(events)?;

        sqlx::query(
            r#"
            INSERT INTO webhooks (id, url, events, secret, is_active, created_at)
            VALUES (?, ?, ?, ?, 1, ?)
            "#
        )
        .bind(&id)
        .bind(url)
        .bind(&events_json)
        .bind(secret)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(crate::models::Webhook {
            id,
            url: url.to_string(),
            events: events.to_vec(),
            secret: secret.map(|s| s.to_string()),
            is_active: true,
            created_at,
            last_triggered_at: None,
        })
    }

    pub async fn get_all_webhooks(&self) -> Result<Vec<crate::models::Webhook>> {
        let rows = sqlx::query_as::<_, (String, String, String, Option<String>, bool, String, Option<String>)>(
            "SELECT id, url, events, secret, is_active, created_at, last_triggered_at FROM webhooks ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, url, events_json, secret, is_active, created_at_str, last_triggered_at_str)| {
            let events: Vec<String> = serde_json::from_str(&events_json).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap()
                .with_timezone(&chrono::Utc);
            let last_triggered_at = last_triggered_at_str.map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .unwrap()
                    .with_timezone(&chrono::Utc)
            });

            crate::models::Webhook {
                id,
                url,
                events,
                secret,
                is_active,
                created_at,
                last_triggered_at,
            }
        }).collect())
    }

    pub async fn get_active_webhooks_for_event(&self, event_type: &str) -> Result<Vec<crate::models::Webhook>> {
        // Get all active webhooks and filter those that listen to this event
        let all_webhooks = self.get_all_webhooks().await?;
        Ok(all_webhooks.into_iter()
            .filter(|w| w.is_active && w.events.contains(&event_type.to_string()))
            .collect())
    }

    pub async fn update_webhook_last_triggered(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE webhooks SET last_triggered_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM webhooks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: String,
    name: String,
    color: Option<String>,
    category: Option<String>,
    created_at: String,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Tag {
            id: row.id,
            name: row.name,
            color: row.color,
            category: row.category,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&chrono::Utc),
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
        DownloadStatus::Converting => "converting".to_string(),
    }
}

fn string_to_status(s: &str) -> DownloadStatus {
    match s {
        "pending" => DownloadStatus::Pending,
        "downloading" => DownloadStatus::Downloading,
        "processing" => DownloadStatus::Processing,
        "completed" => DownloadStatus::Completed,
        "failed" => DownloadStatus::Failed,
        "converting" => DownloadStatus::Converting,
        _ => DownloadStatus::Pending,
    }
}


