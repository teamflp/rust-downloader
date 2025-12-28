use serde::{Deserialize, Serialize};
use tokio::process::Command;
use std::process::Stdio;
use anyhow::{Result, Context};
use log::{info, warn};

/// Metadata structure for video information from yt-dlp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<u64>, // in seconds
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub view_count: Option<u64>,
    pub description: Option<String>,
    pub formats: Option<Vec<FormatInfo>>,
    pub subtitles: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format_id: String,
    pub resolution: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub fps: Option<f32>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
    pub filesize: Option<u64>,
    pub quality: Option<f32>,
}

/// Fetch video metadata using yt-dlp --dump-json
pub async fn get_video_info(url: &str, cookies_browser: Option<&str>) -> Result<VideoInfo> {
    let mut command = Command::new("yt-dlp");
    
    // Get JSON dump of video info
    command.args(&[
        "--dump-json",
        "--no-playlist", // Only get single video info
    ]);
    
    if let Some(browser) = cookies_browser {
        command.args(&["--cookies-from-browser", browser]);
    }
    
    command.arg(url);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    
    info!("Fetching video info for: {}", url);
    
    let output = command
        .output()
        .await
        .context("Failed to execute yt-dlp")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("yt-dlp error: {}", stderr);
        anyhow::bail!("Failed to get video info: {}", stderr);
    }
    
    let json_output = String::from_utf8(output.stdout)
        .context("Failed to parse yt-dlp output as UTF-8")?;
    
    // Parse the JSON response from yt-dlp
    let raw_info: serde_json::Value = serde_json::from_str(&json_output)
        .context("Failed to parse yt-dlp JSON output")?;
    
    // Extract formats for quality selection
    let formats = raw_info.get("formats")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|fmt| {
                    serde_json::from_value::<FormatInfo>(fmt.clone()).ok()
                })
                .collect::<Vec<_>>()
        });
    
    // Build VideoInfo from raw JSON
    Ok(VideoInfo {
        id: raw_info.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        title: raw_info.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        thumbnail: raw_info.get("thumbnail")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        duration: raw_info.get("duration")
            .and_then(|v| v.as_f64())
            .map(|d| d as u64),
        uploader: raw_info.get("uploader")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        uploader_id: raw_info.get("uploader_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        view_count: raw_info.get("view_count")
            .and_then(|v| v.as_u64()),
        description: raw_info.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        formats,
        subtitles: raw_info.get("subtitles").cloned(),
    })
}

