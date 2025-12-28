use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};
use anyhow::{Result, Context};
use tracing::{info, error};

/// Supported conversion formats
#[derive(Debug, Clone, Copy)]
pub enum ConversionFormat {
    Mp4,
    WebM,
    Mkv,
    Mp3,
    Wav,
    Flac,
    M4A,
    Aac,
}

impl ConversionFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ConversionFormat::Mp4 => "mp4",
            ConversionFormat::WebM => "webm",
            ConversionFormat::Mkv => "mkv",
            ConversionFormat::Mp3 => "mp3",
            ConversionFormat::Wav => "wav",
            ConversionFormat::Flac => "flac",
            ConversionFormat::M4A => "m4a",
            ConversionFormat::Aac => "aac",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mp4" => Some(ConversionFormat::Mp4),
            "webm" => Some(ConversionFormat::WebM),
            "mkv" => Some(ConversionFormat::Mkv),
            "mp3" => Some(ConversionFormat::Mp3),
            "wav" => Some(ConversionFormat::Wav),
            "flac" => Some(ConversionFormat::Flac),
            "m4a" => Some(ConversionFormat::M4A),
            "aac" => Some(ConversionFormat::Aac),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn is_video(&self) -> bool {
        matches!(self, ConversionFormat::Mp4 | ConversionFormat::WebM | ConversionFormat::Mkv)
    }

    #[allow(dead_code)]
    pub fn is_audio(&self) -> bool {
        matches!(self, ConversionFormat::Mp3 | ConversionFormat::Wav | ConversionFormat::Flac | ConversionFormat::M4A | ConversionFormat::Aac)
    }
}

/// Get file extension from path
fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Detect if file is video or audio based on extension
#[allow(dead_code)]
fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = get_extension(path) {
        matches!(ext.as_str(), "mp4" | "webm" | "mkv" | "avi" | "mov" | "flv" | "m4v")
    } else {
        false
    }
}

/// Convert a media file to another format using ffmpeg
pub async fn convert_file(
    input_path: &Path,
    output_format: ConversionFormat,
    progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
) -> Result<PathBuf> {
    if !input_path.exists() {
        anyhow::bail!("Input file does not exist: {}", input_path.display());
    }

    // Determine output path
    let output_path = {
        let mut output = input_path.to_path_buf();
        output.set_extension(output_format.extension());
        
        // If output format is same as input, add suffix
        if get_extension(input_path) == Some(output_format.extension().to_string()) {
            let stem = output.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("converted");
            output.set_file_name(format!("{}_{}.{}", stem, output_format.extension(), output_format.extension()));
        }
        
        output
    };

    info!("Converting {} to {}", input_path.display(), output_path.display());

    let mut command = Command::new("ffmpeg");
    
    // Input file
    command.arg("-i").arg(input_path);
    
    // Overwrite output file if exists
    command.arg("-y");
    
    // Copy metadata
    command.arg("-map_metadata").arg("0");
    
    // Format-specific options
    match output_format {
        ConversionFormat::Mp4 => {
            command.args(&["-c:v", "libx264"])
                   .args(&["-c:a", "aac"])
                   .args(&["-movflags", "+faststart"]);
        }
        ConversionFormat::WebM => {
            command.args(&["-c:v", "libvpx-vp9"])
                   .args(&["-c:a", "libopus"])
                   .arg("-b:v").arg("0")
                   .arg("-crf").arg("30");
        }
        ConversionFormat::Mkv => {
            command.args(&["-c:v", "copy"])
                   .args(&["-c:a", "copy"]);
        }
        ConversionFormat::Mp3 => {
            command.args(&["-vn"]) // No video
                   .args(&["-c:a", "libmp3lame"])
                   .arg("-b:a").arg("320k");
        }
        ConversionFormat::Wav => {
            command.args(&["-vn"]) // No video
                   .args(&["-c:a", "pcm_s16le"]);
        }
        ConversionFormat::Flac => {
            command.args(&["-vn"]) // No video
                   .args(&["-c:a", "flac"]);
        }
        ConversionFormat::M4A => {
            command.args(&["-vn"]) // No video
                   .args(&["-c:a", "aac"])
                   .arg("-b:a").arg("256k");
        }
        ConversionFormat::Aac => {
            command.args(&["-vn"]) // No video
                   .args(&["-c:a", "aac"])
                   .arg("-b:a").arg("256k");
        }
    }
    
    // Output file
    command.arg(&output_path);
    
    // Suppress banner and show progress
    command.args(&["-hide_banner", "-progress", "pipe:1", "-loglevel", "error"]);

    let mut child = match command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            error!("Failed to spawn ffmpeg process: {}. Is ffmpeg installed?", e);
            anyhow::bail!("Failed to start ffmpeg: {}. Make sure ffmpeg is installed and in your PATH.", e);
        }
    };

    // Parse progress from stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        // If we have a progress callback, parse duration and time
        if let Some(callback) = progress_callback {
            let mut duration: Option<f64> = None;
            
            while let Ok(Some(line)) = lines.next_line().await {
                if line.starts_with("duration=") {
                    if let Ok(dur) = line.split('=').nth(1).unwrap_or("0").parse::<f64>() {
                        duration = Some(dur);
                    }
                } else if line.starts_with("out_time_ms=") {
                    if let (Some(dur), Some(ms_str)) = (duration, line.split('=').nth(1)) {
                        if let Ok(ms) = ms_str.parse::<f64>() {
                            let seconds = ms / 1_000_000.0;
                            let progress = (seconds / dur * 100.0).min(100.0);
                            callback(progress as f32);
                        }
                    }
                }
            }
        } else {
            // Just consume the output
            while let Ok(Some(_)) = lines.next_line().await {}
        }
    }

    // Wait for process to complete
    let status = child.wait().await.context("Failed to wait for ffmpeg process")?;

    if !status.success() {
        let stderr = if let Some(mut stderr) = child.stderr {
            let mut output = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut output).await;
            output
        } else {
            String::new()
        };
        let exit_code = status.code().unwrap_or(-1);
        error!("ffmpeg conversion failed with exit code {}: {}", exit_code, stderr);
        anyhow::bail!("ffmpeg conversion failed (exit code {}): {}", exit_code, stderr);
    }

    if !output_path.exists() {
        anyhow::bail!("Output file was not created");
    }

    info!("Conversion completed: {}", output_path.display());
    Ok(output_path)
}

/// Get available conversion formats based on input file type
#[allow(dead_code)]
pub fn get_available_formats(input_path: &Path) -> Vec<ConversionFormat> {
    if is_video_file(input_path) {
        vec![
            ConversionFormat::Mp4,
            ConversionFormat::WebM,
            ConversionFormat::Mkv,
            ConversionFormat::Mp3, // Extract audio
        ]
    } else {
        vec![
            ConversionFormat::Mp3,
            ConversionFormat::Wav,
            ConversionFormat::Flac,
            ConversionFormat::M4A,
            ConversionFormat::Aac,
        ]
    }
}

