use axum::{
    extract::State,
    response::{Json, Response},
    http::{StatusCode, header},
};
use crate::state::AppState;
use rust_media_downloader_shared::config;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct ConfigInfo {
    pub download_directory: String,
    pub download_directory_exists: bool,
    pub download_directory_absolute: String,
    pub config_file_path: String,
}

pub async fn get_config_info(
    State(_state): State<AppState>,
) -> Json<ConfigInfo> {
    let config = config::load_config();
    let download_dir = PathBuf::from(&config.download_directory);
    
    // Resolve absolute path
    let download_dir_absolute = if download_dir.is_absolute() {
        download_dir.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&download_dir)
            .canonicalize()
            .unwrap_or_else(|_| {
                // If canonicalize fails, try to construct absolute path manually
                let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
                PathBuf::from(home).join("Downloads")
            })
    };
    
    // Get config file path - use the same logic as config module
    let config_file_path = {
        // Try to get config dir, fallback to a default path
        let config_dir = std::env::var("HOME")
            .map(|home| PathBuf::from(home).join(".config"))
            .unwrap_or_else(|_| PathBuf::from("."));
        config_dir
            .join("rust-media-downloader")
            .join("config.toml")
            .to_string_lossy()
            .to_string()
    };
    
    Json(ConfigInfo {
        download_directory: config.download_directory.clone(),
        download_directory_exists: download_dir_absolute.exists(),
        download_directory_absolute: download_dir_absolute.to_string_lossy().to_string(),
        config_file_path,
    })
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub total_downloads_size: u64, // Taille totale des téléchargements en bytes
    pub total_downloads_count: u32, // Nombre de téléchargements avec fichier
    pub disk_free: Option<u64>, // Espace disque libre en bytes (si disponible)
    pub disk_total: Option<u64>, // Espace disque total en bytes (si disponible)
    pub disk_used_percentage: Option<f64>, // Pourcentage d'utilisation (si disponible)
    pub download_directory: String,
}

pub async fn get_disk_info(
    State(state): State<AppState>,
) -> Json<DiskInfo> {
    let config = config::load_config();
    let download_dir = PathBuf::from(&config.download_directory);
    
    // Resolve absolute path
    let download_dir_absolute = if download_dir.is_absolute() {
        download_dir.clone()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&download_dir)
            .canonicalize()
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
                PathBuf::from(home).join("Downloads")
            })
    };
    
    // Calculate total size from database (sum of file_size for completed downloads)
    let downloads = state.get_all_downloads().await;
    let (total_size, count) = downloads
        .iter()
        .filter(|d| d.status == crate::models::DownloadStatus::Completed && d.file_size.is_some())
        .fold((0u64, 0u32), |(size, cnt), d| {
            (size + d.file_size.unwrap_or(0), cnt + 1)
        });
    
    // Try to get disk space information
    let (disk_free, disk_total, disk_used_percentage) = get_disk_space(&download_dir_absolute);
    
    Json(DiskInfo {
        total_downloads_size: total_size,
        total_downloads_count: count,
        disk_free,
        disk_total,
        disk_used_percentage,
        download_directory: download_dir_absolute.to_string_lossy().to_string(),
    })
}

fn get_disk_space(path: &PathBuf) -> (Option<u64>, Option<u64>, Option<f64>) {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem;
        
        extern "C" {
            fn statvfs(path: *const libc::c_char, buf: *mut libc::statvfs) -> libc::c_int;
        }
        
        unsafe {
            let path_str = path.to_string_lossy().to_string();
            if let Ok(c_path) = CString::new(path_str) {
                let mut stat: libc::statvfs = mem::zeroed();
                if statvfs(c_path.as_ptr(), &mut stat) == 0 {
                    let block_size = stat.f_frsize as u64;
                    let total = stat.f_blocks as u64 * block_size;
                    let free = stat.f_bavail as u64 * block_size;
                    let used = total - free;
                    let used_percentage = if total > 0 {
                        Some((used as f64 / total as f64) * 100.0)
                    } else {
                        None
                    };
                    return (Some(free), Some(total), used_percentage);
                }
            }
        }
    }
    
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::mem;
        
        unsafe {
            let path_wide: Vec<u16> = path.as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            
            let mut free_bytes: u64 = 0;
            let mut total_bytes: u64 = 0;
            
            // Use GetDiskFreeSpaceExW through winapi
            if winapi::um::fileapi::GetDiskFreeSpaceExW(
                path_wide.as_ptr(),
                &mut free_bytes as *mut _ as *mut winapi::um::winnt::ULARGE_INTEGER,
                &mut total_bytes as *mut _ as *mut winapi::um::winnt::ULARGE_INTEGER,
                std::ptr::null_mut(),
            ) != 0 {
                let used = total_bytes - free_bytes;
                let used_percentage = if total_bytes > 0 {
                    Some((used as f64 / total_bytes as f64) * 100.0)
                } else {
                    None
                };
                return (Some(free_bytes), Some(total_bytes), used_percentage);
            }
        }
    }
    
    // Fallback: return None if platform not supported or call failed
    (None, None, None)
}

/// Serve the DISCLAIMER.md file
pub async fn get_disclaimer() -> Response {
    // Read DISCLAIMER.md from the project root (embedded at compile time)
    // Path: backend/src/api/config.rs -> ../../../DISCLAIMER.md (3 levels up to project root)
    let disclaimer_content = include_str!("../../../DISCLAIMER.md");
    
    tracing::info!("Serving DISCLAIMER.md ({} bytes)", disclaimer_content.len());
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(axum::body::Body::from(disclaimer_content))
        .unwrap()
}

/// Serve the LICENSE file
pub async fn get_license() -> Response {
    // Read LICENSE from the project root (embedded at compile time)
    // Path: backend/src/api/config.rs -> ../../../LICENSE (3 levels up to project root)
    let license_content = include_str!("../../../LICENSE");
    
    tracing::info!("Serving LICENSE ({} bytes)", license_content.len());
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(axum::body::Body::from(license_content))
        .unwrap()
}

