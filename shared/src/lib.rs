pub mod downloader;
pub mod spleeter;
pub mod config;
pub mod cookies;
pub mod commands;
pub mod installers;
pub mod utils;
pub mod progress;

// Re-export commonly used items
pub use downloader::{download_video, download_audio};
pub use spleeter::extract_instrumental;
pub use config::{Config, load_config, save_config};
pub use cookies::extract_cookies_and_download;
pub use commands::check_command;
pub use installers::ensure_dependencies;
