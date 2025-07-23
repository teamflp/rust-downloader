use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub default_audio_format: String,
    pub audio_formats: Vec<String>,
    pub default_video_format: String,
    pub download_directory: String,
    pub keep_temporary_files: bool,
}

impl Default for Config {
    fn default() -> Self {
        let download_directory = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .to_str()
            .unwrap_or(".")
            .to_string();

        Config {
            default_audio_format: "mp3".to_string(),
            audio_formats: vec!["mp3".to_string(), "m4a".to_string(), "flac".to_string(), "wav".to_string()],
            default_video_format: "mp4".to_string(),
            download_directory,
            keep_temporary_files: false,
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().expect("Could not find config directory");
    path.push("rust-media-downloader");
    fs::create_dir_all(&path).expect("Could not create config directory");
    path.push("config.toml");
    path
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if !path.exists() {
        let config = Config::default();
        save_config(&config);
        return config;
    }

    let content = fs::read_to_string(&path).expect("Could not read config file");
    match toml::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            log::warn!("Failed to parse config file, it might be outdated: {}. Creating a new one.", e);
            // Attempt to remove the old, invalid config file
            let _ = fs::remove_file(&path);
            let config = Config::default();
            save_config(&config);
            config
        }
    }
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    let content = toml::to_string(config).expect("Could not serialize config");
    fs::write(path, content).expect("Could not write to config file");
}