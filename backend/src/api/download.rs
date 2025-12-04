use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use crate::{
    models::{DownloadRequest, DownloadResponse, DownloadStatus, DownloadType, ErrorResponse},
    state::AppState,
    validation::validate_url,
};
use rust_media_downloader_shared::{download_video, download_audio, extract_instrumental};

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
            state_clone.update_download(&download_id, dl).await;
        }

        // Perform the actual download
        let result = perform_download(request_clone, download_id.clone()).await;
        
        // Update download status based on result
        if let Some(mut dl) = state_clone.get_download(&download_id).await {
            match result {
                Ok(file_path) => {
                    dl.set_status(DownloadStatus::Completed, "Download completed successfully".to_string());
                    dl.file_path = Some(file_path);
                    dl.progress = 100.0;
                }
                Err(e) => {
                    dl.set_status(DownloadStatus::Failed, format!("Download failed: {}", e));
                }
            }
            state_clone.update_download(&download_id, dl).await;
        }
    });

    Ok(Json(download))
}

async fn perform_download(request: DownloadRequest, _id: String) -> anyhow::Result<String> {
    let download_playlist = request.download_playlist.unwrap_or(false);
    
    match request.download_type {
        DownloadType::Video => {
            let format = request.format.as_deref().unwrap_or("mp4");
            download_video(
                &request.url,
                format,
                false,
                request.custom_filename,
                request.cookies_browser,
                download_playlist,
            ).await?;
            Ok(format!("video.{}", format))
        }
        DownloadType::Audio => {
            let format = request.format.as_deref().unwrap_or("mp3");
            download_audio(
                &request.url,
                format,
                false,
                request.custom_filename,
                request.cookies_browser,
                download_playlist,
            ).await?;
            Ok(format!("audio.{}", format))
        }
        DownloadType::Instrumental => {
            let format = request.format.as_deref().unwrap_or("mp3");
            // First download audio
            download_audio(
                &request.url,
                format,
                true,
                request.custom_filename.clone(),
                request.cookies_browser,
                download_playlist,
            ).await?;
            
            // Then extract instrumental
            let audio_file = std::path::PathBuf::from(format!("audio.{}", format));
            extract_instrumental(&audio_file).await?;
            Ok(format!("instrumental.{}", format))
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
) -> Json<Vec<DownloadResponse>> {
    let downloads = state.get_all_downloads().await;
    Json(downloads)
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
