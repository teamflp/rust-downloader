use super::config;
use std::fs;

#[test]
fn test_load_and_save_config() {
    let mut config = config::load_config();
    config.default_audio_format = "test_audio".to_string();
    config.audio_formats.push("test_audio".to_string());
    config.default_video_format = "test_video".to_string();
    config.download_directory = "/tmp/test_dir".to_string();
    config.keep_temporary_files = true;
    config::save_config(&config);

    let loaded_config = config::load_config();
    assert_eq!(loaded_config.default_audio_format, "test_audio");
    assert!(loaded_config.audio_formats.contains(&"test_audio".to_string()));
    assert_eq!(loaded_config.default_video_format, "test_video");
    assert_eq!(loaded_config.download_directory, "/tmp/test_dir");
    assert_eq!(loaded_config.keep_temporary_files, true);

    // Cleanup
    let mut path = dirs::config_dir().expect("Could not find config directory");
    path.push("rust-media-downloader");
    path.push("config.toml");
    fs::remove_file(path).expect("Could not remove test config file");
}
