pub mod download;
pub mod video;
pub mod logs;
pub mod files;
pub mod config;
pub mod tags;
pub mod statistics;
pub mod webhooks;

pub use download::{create_download, create_batch_downloads, get_download, list_downloads, get_all_downloads, delete_download, update_metadata, convert_download, toggle_favorite, export_downloads, import_downloads};
pub use video::get_video_info_endpoint;
pub use logs::get_logs;
pub use files::serve_file;
pub use config::{get_config_info, get_disk_info, get_disclaimer, get_license};
pub use tags::{create_tag, get_tag, list_tags, update_tag, delete_tag, get_download_tags, add_tag_to_download, remove_tag_from_download, set_download_tags};
pub use statistics::get_statistics;
pub use webhooks::{create_webhook, list_webhooks, delete_webhook};
