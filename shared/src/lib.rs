pub mod downloader;
pub mod spleeter;
pub mod config;
pub mod cookies;
pub mod commands;
pub mod installers;
pub mod utils;
pub mod progress;
pub mod video_info;

// Re-export commonly used items
pub use downloader::{download_video, download_audio};
pub use spleeter::extract_instrumental;
pub use config::{Config, load_config, save_config, get_config_path};
pub use cookies::extract_cookies_and_download;
pub use commands::check_command;
pub use installers::ensure_dependencies;
pub use video_info::{VideoInfo, get_video_info};
