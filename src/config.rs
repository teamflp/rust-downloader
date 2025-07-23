use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub default_audio_format: String,
    pub audio_formats: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_audio_format: "mp3".to_string(),
            audio_formats: vec!["mp3".to_string(), "m4a".to_string(), "flac".to_string(), "wav".to_string()],
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

    let content = fs::read_to_string(path).expect("Could not read config file");
    toml::from_str(&content).expect("Could not parse config file")
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    let content = toml::to_string(config).expect("Could not serialize config");
    fs::write(path, content).expect("Could not write to config file");
}
