use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{Json, Response},
    body::Body,
};
use crate::{
    models::{DownloadRequest, DownloadResponse, DownloadStatus, DownloadType, ErrorResponse, PaginatedResponse, PaginationParams, UpdateMetadataRequest, ConvertFileRequest},
    state::AppState,
    validation::validate_url,
};
use rust_media_downloader_shared::{download_video_enhanced, download_audio_enhanced};
use utoipa;

#[utoipa::path(
    post,
    path = "/api/downloads",
    request_body = DownloadRequest,
    responses(
        (status = 200, description = "Download created successfully", body = DownloadResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "downloads",
)]
pub async fn create_download(
    State(state): State<AppState>,
    Json(request): Json<DownloadRequest>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate URL
    if let Err(e) = validate_url(&request.url) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("validation_error", e.message)),
        ));
    }

    // Create download response
    let download = DownloadResponse::new(request.url.clone(), request.download_type.clone());
    let download_id = download.id.clone();
    
    // Add to state
    state.add_download(download.clone()).await;

    // Spawn background task for actual download
    let state_clone = state.clone();
    let request_clone = request.clone();
    tokio::spawn(async move {
        // Update status to Downloading
        if let Some(mut dl) = state_clone.get_download(&download_id).await {
            dl.set_status(
                DownloadStatus::Downloading, 
                "Downloading from YouTube... This may take 30-60 seconds depending on video size. Please wait.".to_string()
            );
            dl.progress = 10.0; // Start at 10% when downloading begins
            state_clone.update_download(&download_id, dl).await;
        }

        // Spawn a task to update progress approximately every 2 seconds
        let state_progress = state_clone.clone();
        let download_id_progress = download_id.clone();
        let progress_handle = tokio::spawn(async move {
            let mut elapsed = 0u64;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                elapsed += 2;
                
                // Simulate progress: start at 10%, gradually increase to 90% over ~60 seconds
                // This gives a reasonable approximation for most videos
                let current_progress = if elapsed < 60 {
                    10.0 + (elapsed as f32 / 60.0) * 80.0 // 10% to 90% over 60 seconds
                } else {
                    90.0 // Cap at 90% until actual completion
                };
                
                if let Some(mut dl) = state_progress.get_download(&download_id_progress).await {
                    // Only update if still downloading
                    if dl.status == DownloadStatus::Downloading {
                        dl.progress = current_progress;
                        state_progress.update_download(&download_id_progress, dl).await;
                    } else {
                        // Download finished, stop updating progress
                        break;
                    }
                } else {
                    // Download not found, stop
                    break;
                }
            }
        });

        // Perform the actual download
        let result = perform_download(request_clone, download_id.clone()).await;
        
        // Cancel progress update task
        progress_handle.abort();
        
        // Update download status based on result
        if let Some(mut dl) = state_clone.get_download(&download_id).await {
            match result {
                Ok(file_path) => {
                    dl.set_status(DownloadStatus::Completed, "Download completed successfully".to_string());
                    dl.file_path = Some(file_path.clone());
                    dl.progress = 100.0;
                    let dl_clone = dl.clone();
                    state_clone.update_download(&download_id, dl).await;
                    
                    // Trigger webhook for completed download
                    let state_webhook = state_clone.clone();
                    tokio::spawn(async move {
                        let event_data = serde_json::json!({
                            "download_id": download_id,
                            "status": "completed",
                            "file_path": dl_clone.file_path,
                            "url": dl_clone.url,
                            "title": dl_clone.title,
                        });
                        crate::api::webhooks::trigger_webhooks(&state_webhook, "download.completed", event_data).await;
                    });
                }
                Err(e) => {
                    let error_msg = format!("Download failed: {}", e);
                    dl.set_status(DownloadStatus::Failed, error_msg.clone());
                    let dl_clone = dl.clone();
                    state_clone.update_download(&download_id, dl).await;
                    
                    // Trigger webhook for failed download
                    let state_webhook = state_clone.clone();
                    tokio::spawn(async move {
                        let event_data = serde_json::json!({
                            "download_id": download_id,
                            "status": "failed",
                            "error": error_msg,
                            "url": dl_clone.url,
                            "title": dl_clone.title,
                        });
                        crate::api::webhooks::trigger_webhooks(&state_webhook, "download.failed", event_data).await;
                    });
                }
            }
        }
    });

    Ok(Json(download))
}

pub async fn create_batch_downloads(
    State(state): State<AppState>,
    Json(requests): Json<Vec<DownloadRequest>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let mut created = 0;
    let mut errors = 0;
    let mut error_details = Vec::new();
    
    for request in requests {
        // Validate URL
        if let Err(e) = validate_url(&request.url) {
            errors += 1;
            error_details.push(serde_json::json!({
                "url": request.url,
                "error": e.message
            }));
            continue;
        }

        // Create download response
        let download = DownloadResponse::new(request.url.clone(), request.download_type.clone());
        let download_id = download.id.clone();
        
        // Add to state
        state.add_download(download.clone()).await;

        // Spawn background task for actual download
        let state_clone = state.clone();
        let request_clone = request.clone();
        tokio::spawn(async move {
            // Update status to Downloading
            if let Some(mut dl) = state_clone.get_download(&download_id).await {
                dl.set_status(
                    DownloadStatus::Downloading, 
                    "Downloading from YouTube... This may take 30-60 seconds depending on video size. Please wait.".to_string()
                );
                dl.progress = 10.0;
                state_clone.update_download(&download_id, dl).await;
            }

            // Spawn a task to update progress approximately every 2 seconds
            let state_progress = state_clone.clone();
            let download_id_progress = download_id.clone();
            let progress_handle = tokio::spawn(async move {
                let mut elapsed = 0u64;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    elapsed += 2;
                    
                    let current_progress = if elapsed < 60 {
                        10.0 + (elapsed as f32 / 60.0) * 80.0
                    } else {
                        90.0
                    };
                    
                    if let Some(mut dl) = state_progress.get_download(&download_id_progress).await {
                        if dl.status == DownloadStatus::Downloading {
                            dl.progress = current_progress;
                            state_progress.update_download(&download_id_progress, dl).await;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            });

            // Perform actual download
            match perform_download(request_clone, download_id.clone()).await {
                Ok(file_path) => {
                    progress_handle.abort();
                    if let Some(mut dl) = state_clone.get_download(&download_id).await {
                        dl.status = DownloadStatus::Completed;
                        dl.progress = 100.0;
                        dl.message = "Download completed successfully".to_string();
                        dl.file_path = Some(file_path.clone());
                        dl.completed_at = Some(chrono::Utc::now());
                        let dl_clone = dl.clone();
                        state_clone.update_download(&download_id, dl).await;
                        
                        // Trigger webhook
                        let state_webhook = state_clone.clone();
                        let download_id_webhook = download_id.clone();
                        tokio::spawn(async move {
                            let event_data = serde_json::json!({
                                "download_id": download_id_webhook,
                                "status": "completed",
                                "file_path": dl_clone.file_path,
                                "url": dl_clone.url,
                                "title": dl_clone.title,
                            });
                            crate::api::webhooks::trigger_webhooks(&state_webhook, "download.completed", event_data).await;
                        });
                    }
                }
                Err(e) => {
                    progress_handle.abort();
                    tracing::error!("Download failed for {}: {}", download_id, e);
                    if let Some(mut dl) = state_clone.get_download(&download_id).await {
                        dl.status = DownloadStatus::Failed;
                        let error_msg = format!("Download failed: {}", e);
                        dl.message = error_msg.clone();
                        let dl_clone = dl.clone();
                        state_clone.update_download(&download_id, dl).await;
                        
                        // Trigger webhook
                        let state_webhook = state_clone.clone();
                        let download_id_webhook = download_id.clone();
                        tokio::spawn(async move {
                            let event_data = serde_json::json!({
                                "download_id": download_id_webhook,
                                "status": "failed",
                                "error": error_msg,
                                "url": dl_clone.url,
                                "title": dl_clone.title,
                            });
                            crate::api::webhooks::trigger_webhooks(&state_webhook, "download.failed", event_data).await;
                        });
                    }
                }
            }
        });

        created += 1;
    }

    Ok(Json(serde_json::json!({
        "created": created,
        "errors": errors,
        "error_details": error_details,
        "total": created + errors
    })))
}

async fn perform_download(request: DownloadRequest, _id: String) -> anyhow::Result<String> {
    let download_playlist = request.download_playlist.unwrap_or(false);
    let download_subtitles = request.download_subtitles.unwrap_or(false);
    let subtitle_language = request.subtitle_language.as_deref();
    
    match request.download_type {
        DownloadType::Video => {
            let format = request.format.as_deref().unwrap_or("mp4");
            let resolution = request.resolution.as_deref();
            let audio_quality = request.audio_quality.as_deref();
            
            let file_path = download_video_enhanced(
                &request.url,
                format,
                resolution,
                audio_quality,
                download_subtitles,
                subtitle_language,
                false,
                request.custom_filename,
                request.cookies_browser,
                download_playlist,
            ).await?;
            Ok(file_path)
        }
        DownloadType::Audio => {
            let format = request.format.as_deref().unwrap_or("mp3");
            let audio_quality = request.audio_quality.as_deref();
            
            let file_path = download_audio_enhanced(
                &request.url,
                format,
                audio_quality,
                false,
                request.custom_filename,
                request.cookies_browser,
                download_playlist,
            ).await?;
            Ok(file_path)
        }
        DownloadType::Instrumental => {
            let format = request.format.as_deref().unwrap_or("mp3");
            let audio_quality = request.audio_quality.as_deref();
            
            // First download audio and extract instrumental
            let file_path = download_audio_enhanced(
                &request.url,
                format,
                audio_quality,
                true, // extract_instrumental = true
                request.custom_filename.clone(),
                request.cookies_browser,
                download_playlist,
            ).await?;
            Ok(file_path)
        }
    }
}

pub async fn get_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.get_download(&id).await {
        Some(download) => Ok(Json(download)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found")),
        )),
    }
}

pub async fn list_downloads(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Json<PaginatedResponse<DownloadResponse>> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    
    let (downloads, total) = state.get_downloads_paginated(page, per_page).await;
    let total_pages = (total as f64 / per_page as f64).ceil() as u32;
    
    Json(PaginatedResponse {
        items: downloads,
        total,
        page,
        per_page,
        total_pages,
    })
}

pub async fn get_all_downloads(
    State(state): State<AppState>,
) -> Json<Vec<DownloadResponse>> {
    Json(state.get_all_downloads().await)
}

pub async fn delete_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match state.remove_download(&id).await {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found")),
        )),
    }
}

pub async fn update_metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMetadataRequest>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Verify download exists
    if state.get_download(&id).await.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found")),
        ));
    }

    // Update metadata
    if let Err(e) = state.update_metadata(
        &id,
        request.title.as_deref(),
        request.author.as_deref(),
        request.notes.as_deref(),
    ).await {
        tracing::error!("Failed to update metadata for download {}: {}", id, e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("update_failed", format!("Failed to update metadata: {}", e))),
        ));
    }

    // Get updated download
    match state.get_download(&id).await {
        Some(updated) => Ok(Json(updated)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found after update")),
        )),
    }
}

#[derive(serde::Deserialize)]
pub struct ExportParams {
    format: Option<String>,
}

pub async fn export_downloads(
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let format = params.format.as_deref().unwrap_or("json").to_lowercase();
    
    let downloads = state.get_all_downloads().await;
    
    match format.as_str() {
        "json" => {
            let json_data = serde_json::to_string_pretty(&downloads)
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new("serialization_error", format!("Failed to serialize downloads: {}", e))),
                    )
                })?;
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"downloads_export.json\"")
                .body(Body::from(json_data))
                .unwrap())
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            
            // Write header
            wtr.write_record(&[
                "id", "url", "type", "status", "progress", "message",
                "created_at", "completed_at", "file_path", "is_playlist",
                "total_items", "completed_items", "title", "thumbnail",
                "duration", "author", "file_size", "retry_count", "max_retries", "notes"
            ]).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("csv_error", format!("Failed to write CSV header: {}", e))),
                )
            })?;
            
            // Write data
            for download in downloads {
                wtr.write_record(&[
                    download.id.clone(),
                    download.url.clone(),
                    format!("{:?}", download.download_type).to_lowercase(),
                    format!("{:?}", download.status).to_lowercase(),
                    download.progress.to_string(),
                    download.message.clone(),
                    download.created_at.to_rfc3339(),
                    download.completed_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                    download.file_path.clone().unwrap_or_default(),
                    download.is_playlist.to_string(),
                    download.total_items.map(|i| i.to_string()).unwrap_or_default(),
                    download.completed_items.map(|i| i.to_string()).unwrap_or_default(),
                    download.title.clone().unwrap_or_default(),
                    download.thumbnail.clone().unwrap_or_default(),
                    download.duration.map(|d| d.to_string()).unwrap_or_default(),
                    download.author.clone().unwrap_or_default(),
                    download.file_size.map(|s| s.to_string()).unwrap_or_default(),
                    download.retry_count.map(|r| r.to_string()).unwrap_or_default(),
                    download.max_retries.map(|r| r.to_string()).unwrap_or_default(),
                    download.notes.clone().unwrap_or_default(),
                ]).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new("csv_error", format!("Failed to write CSV record: {}", e))),
                    )
                })?;
            }
            
            let csv_data = wtr.into_inner().map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("csv_error", format!("Failed to finalize CSV: {}", e))),
                )
            })?;
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=\"downloads_export.csv\"")
                .body(Body::from(csv_data))
                .unwrap())
        }
        _ => {
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("invalid_format", "Format must be 'json' or 'csv'")),
            ))
        }
    }
}

pub async fn import_downloads(
    State(state): State<AppState>,
    Json(downloads): Json<Vec<DownloadResponse>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let total = downloads.len();
    
    for download in downloads {
        // Check if download already exists
        if state.get_download(&download.id).await.is_some() {
            skipped += 1;
            continue;
        }
        
        // Validate URL
        if crate::validation::validate_url(&download.url).is_err() {
            errors += 1;
            continue;
        }
        
        // Insert download
        state.add_download(download).await;
        
        imported += 1;
    }
    
    Ok(Json(serde_json::json!({
        "imported": imported,
        "skipped": skipped,
        "errors": errors,
        "total": total
    })))
}

pub async fn convert_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<ConvertFileRequest>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    let keep_original = request.keep_original.unwrap_or(false);
    
    if let Err(e) = state.convert_download(&id, &request.format, keep_original).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("conversion_error", format!("Conversion failed: {}", e))),
        ));
    }
    
    match state.get_download(&id).await {
        Some(download) => Ok(Json(download)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found after conversion")),
        )),
    }
}

pub async fn toggle_favorite(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DownloadResponse>, (StatusCode, Json<ErrorResponse>)> {
    if let Err(e) = state.toggle_favorite(&id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("favorite_error", format!("Failed to toggle favorite: {}", e))),
        ));
    }
    
    match state.get_download(&id).await {
        Some(download) => Ok(Json(download)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("not_found", "Download not found")),
        )),
    }
}
