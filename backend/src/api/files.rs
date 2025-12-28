use axum::{
    extract::{Path, State, Request},
    http::{header, StatusCode},
    response::Response,
    body::Body,
};
use crate::state::AppState;
use crate::models::ErrorResponse;
use std::path::PathBuf;
use rust_media_downloader_shared::config;
use tokio_util::io::ReaderStream;
use futures::StreamExt;

/// Parse Range header (e.g., "bytes=0-1023")
fn parse_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_header.starts_with("bytes=") {
        return None;
    }
    
    let range = &range_header[6..]; // Skip "bytes="
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let start = if parts[0].is_empty() {
        // Suffix range: "-500" means last 500 bytes
        let suffix = parts[1].parse::<u64>().ok()?;
        if suffix > file_size {
            return Some((0, file_size - 1));
        }
        file_size - suffix
    } else {
        parts[0].parse::<u64>().ok()?
    };
    
    let end = if parts[1].is_empty() {
        // Open-ended range: "500-" means from byte 500 to end
        file_size - 1
    } else {
        parts[1].parse::<u64>().ok()?
    };
    
    if start > end || end >= file_size {
        return None;
    }
    
    Some((start, end))
}

/// Serve a downloaded file with Range request support
pub async fn serve_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    request: Request,
) -> Result<Response, (StatusCode, axum::Json<ErrorResponse>)> {
    let headers = request.headers();
    // Get download info from database
    let download = match state.get_download(&id).await {
        Some(d) => d,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                axum::Json(ErrorResponse::new("not_found", "Download not found")),
            ));
        }
    };

    // Check if download is completed and has a file path
    if download.status != crate::models::DownloadStatus::Completed {
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(ErrorResponse::new(
                "not_ready",
                "Download is not completed yet",
            )),
        ));
    }

    let file_path = match download.file_path {
        Some(path) => path,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                axum::Json(ErrorResponse::new("no_file", "File path not found")),
            ));
        }
    };

    // Load config to get download directory
    let config = config::load_config();
    let download_dir = PathBuf::from(&config.download_directory);
    
    // Construct full path
    let full_path = if PathBuf::from(&file_path).is_absolute() {
        PathBuf::from(&file_path)
    } else {
        download_dir.join(&file_path)
    };

    tracing::info!("Serving file: {} (full path: {})", file_path, full_path.display());

    // Check if file exists, if not try to find alternative extensions
    let mut actual_path = full_path.clone();
    if !actual_path.exists() {
        // Try to find file with different extensions (common case: .f251.webm -> .mp4 or .webm)
        let parent_opt = actual_path.parent().map(|p| p.to_path_buf());
        let stem_opt = actual_path.file_stem().map(|s| s.to_string_lossy().to_string());
        
        if let (Some(parent), Some(stem_str)) = (parent_opt, stem_opt) {
            // Remove .f251 or similar format codes from stem
            let clean_stem = stem_str
                .replace(".f251", "")
                .replace(".f140", "")
                .replace(".f137", "")
                .replace(".f248", "")
                .replace(".f249", "")
                .replace(".f250", "");
            
            // Try common video/audio extensions
            let extensions = ["mp4", "webm", "mkv", "mp3", "m4a", "wav", "flac"];
            for ext in &extensions {
                let candidate = parent.join(format!("{}.{}", clean_stem, ext));
                if candidate.exists() {
                    tracing::info!("Found alternative file: {} -> {}", full_path.display(), candidate.display());
                    actual_path = candidate;
                    break;
                }
            }
            
            // If still not found, try with original stem but different extensions
            if !actual_path.exists() {
                for ext in &extensions {
                    let candidate = parent.join(format!("{}.{}", stem_str, ext));
                    if candidate.exists() {
                        tracing::info!("Found alternative file with original stem: {} -> {}", full_path.display(), candidate.display());
                        actual_path = candidate;
                        break;
                    }
                }
            }
        }
    }

    // Final check if file exists
    if !actual_path.exists() {
        tracing::error!("File not found: {} (searched at: {})", file_path, actual_path.display());
        return Err((
            StatusCode::NOT_FOUND,
            axum::Json(ErrorResponse::new("file_not_found", &format!("File does not exist on disk: {}. Searched at: {}", file_path, actual_path.display()))),
        ));
    }

    // Determine content type based on file extension (use actual_path, not full_path)
    let content_type = get_content_type(&actual_path);

    // Open file for streaming (use actual_path, not full_path)
    match tokio::fs::File::open(&actual_path).await {
        Ok(file) => {
            // Get file metadata for content length
            let metadata = file.metadata().await.map_err(|e| {
                tracing::error!("Failed to get file metadata {}: {}", full_path.display(), e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(ErrorResponse::new("metadata_error", "Failed to get file metadata")),
                )
            })?;

            let file_size = metadata.len();
            
            // Check for Range request
            let range_header = headers.get(header::RANGE)
                .and_then(|v| v.to_str().ok());
            
            let (start, end, status, content_length) = if let Some(range_str) = range_header {
                if let Some((range_start, range_end)) = parse_range(range_str, file_size) {
                    tracing::info!("Range request: bytes={}-{}", range_start, range_end);
                    let content_len = range_end - range_start + 1;
                    (range_start, range_end, StatusCode::PARTIAL_CONTENT, content_len)
                } else {
                    // Invalid range, return full file
                    (0, file_size - 1, StatusCode::OK, file_size)
                }
            } else {
                // No range request, return full file
                (0, file_size - 1, StatusCode::OK, file_size)
            };
            
            // Reopen file and seek to start position
            let mut file = tokio::fs::File::open(&actual_path).await.map_err(|e| {
                tracing::error!("Failed to reopen file {}: {}", actual_path.display(), e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(ErrorResponse::new("read_error", "Failed to open file")),
                )
            })?;
            
            // Seek to start position if needed
            if start > 0 {
                use tokio::io::{AsyncSeekExt, SeekFrom};
                file.seek(SeekFrom::Start(start)).await.map_err(|e| {
                    tracing::error!("Failed to seek file {}: {}", full_path.display(), e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(ErrorResponse::new("seek_error", "Failed to seek file")),
                    )
                })?;
            }
            
            // Create a limited stream that only reads the requested range
            use tokio::io::AsyncReadExt;
            let limited_file = file.take(content_length);
            let stream = ReaderStream::new(limited_file);
            let body_stream = stream.map(|result| {
                result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            });
            let stream_body = Body::from_stream(body_stream);

            let mut response_builder = Response::builder()
                .status(status)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CONTENT_LENGTH, content_length.to_string())
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Range, Content-Type");
            
            // Add Content-Range header for partial content
            if status == StatusCode::PARTIAL_CONTENT {
                response_builder = response_builder.header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size)
                );
            }
            
            let response = response_builder
                .body(stream_body)
                .map_err(|e| {
                    tracing::error!("Failed to build response: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(ErrorResponse::new("response_error", "Failed to build response")),
                    )
                })?;

            Ok(response)
        }
        Err(e) => {
            tracing::error!("Failed to open file {}: {}", full_path.display(), e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(ErrorResponse::new("read_error", "Failed to open file")),
            ))
        }
    }
}

fn get_content_type(path: &PathBuf) -> &'static str {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        // Video formats
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        // Audio formats
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "opus" => "audio/opus",
        // Default
        _ => "application/octet-stream",
    }
}

