use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use rust_media_downloader_shared::{get_video_info, VideoInfo};
use crate::models::ErrorResponse;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct VideoInfoQuery {
    url: String,
    cookies_browser: Option<String>,
}

pub async fn get_video_info_endpoint(
    State(state): State<AppState>,
    Query(params): Query<VideoInfoQuery>,
) -> Result<Json<VideoInfo>, (StatusCode, Json<ErrorResponse>)> {
    // Validate URL
    if params.url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("validation_error", "URL is required")),
        ));
    }

    // Check cache first
    let cache = state.get_video_cache();
    
    // Clean expired entries periodically (every 10th request approximately)
    use std::sync::atomic::{AtomicU64, Ordering};
    static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
    let count = REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
    if count % 10 == 0 {
        cache.clear_expired().await;
    }
    
    if let Some(cached_info) = cache.get(&params.url).await {
        tracing::debug!("Video info found in cache for: {}", params.url);
        return Ok(Json(cached_info));
    }

    // Fetch from API
    match get_video_info(&params.url, params.cookies_browser.as_deref()).await {
        Ok(info) => {
            // Store in cache
            cache.set(params.url, info.clone()).await;
            Ok(Json(info))
        }
        Err(e) => {
            tracing::error!("Failed to get video info: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("fetch_error", &format!("Failed to fetch video info: {}", e))),
            ))
        }
    }
}

