use rust_media_downloader_shared::config::{load_config, get_config_path};
use std::fs;

fn main() {
    let path = get_config_path();
    println!("Chemin du fichier config: {:?}", path);
    
    if path.exists() {
        let content = fs::read_to_string(&path).expect("Could not read config file");
        println!("\n=== CONTENU BRUT DU FICHIER ===");
        println!("{}", content);
        println!("=== FIN DU CONTENU ===\n");
        
    } else {
        println!("❌ Le fichier de configuration n'existe pas");
    }
    
    let config = load_config();
    println!("\n=== CONFIGURATION CHARGÉE ===");
    println!("Download directory: {}", config.download_directory);
    println!("Default audio format: {}", config.default_audio_format);
    println!("Default video format: {}", config.default_video_format);
}
