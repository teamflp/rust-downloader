use super::config;
use std::fs;

#[test]
fn test_load_and_save_config() {
    let mut config = config::load_config();
    config.default_audio_format = "test".to_string();
    config.audio_formats.push("test".to_string());
    config::save_config(&config);

    let loaded_config = config::load_config();
    assert_eq!(loaded_config.default_audio_format, "test");
    assert!(loaded_config.audio_formats.contains(&"test".to_string()));

    // Cleanup
    let mut path = dirs::config_dir().expect("Could not find config directory");
    path.push("rust-media-downloader");
    path.push("config.toml");
    fs::remove_file(path).expect("Could not remove test config file");
}
