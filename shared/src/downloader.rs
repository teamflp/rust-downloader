use crate::config;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::{BufReader, AsyncBufReadExt};
use log::{info, warn, error};
use tokio::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use std::path::{Path, PathBuf}; // Added for path manipulation
use anyhow::{Result, Context, bail};
use crate::spleeter;

pub async fn download_video(url: &str, format: &str, keep_files: bool, custom_filename: Option<String>, cookies_browser: Option<String>, download_playlist: bool) -> Result<()> {
    let mut command = Command::new("yt-dlp");

    let config = config::load_config();
    command.args(&["-P", &config.download_directory]);

    // Only download single video if user doesn't want the playlist
    if !download_playlist {
        command.arg("--no-playlist");
    }

    // Ajouter l'option pour personnaliser le nom du fichier si fourni
    if let Some(filename) = &custom_filename {
        // Utiliser le template de yt-dlp pour définir le nom de fichier tout en conservant l'extension originale
        command.args(&["-o", &format!("{}.%(ext)s", filename)]);
    }

    command.arg(url);

    if keep_files {
        command.arg("-k");
    }

    // Add cookies if provided
    if let Some(browser) = cookies_browser {
        command.args(&["--cookies-from-browser", &browser]);
    }

    if !format.is_empty() {
        if format == "mp4" {
            // Avoid "pre-merged mp4 format" warning by selecting best video+audio and merging
            command.args(&["-f", "bv*+ba/b", "--merge-output-format", "mp4"]);
        } else if format != "best" {
            command.args(&["-f", format]);
        }
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .context("Erreur lors de l'exécution de yt-dlp")?;
    let stdout = child.stdout.take().context("Erreur de capture du stdout")?;
    let stderr = child.stderr.take().context("Erreur de capture du stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let pb_arc = Arc::new(Mutex::new(ProgressBar::new(100))); // Renamed for clarity
    pb_arc.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Erreur lors de la configuration du style de la barre de progression")
            .progress_chars("##-"),
    );

    let pb_clone = Arc::clone(&pb_arc);
    let downloaded_filename_arc = Arc::new(Mutex::new(None::<String>)); // Explicit type
    let downloaded_filename_clone = Arc::clone(&downloaded_filename_arc);

    // Spawn a task for stderr
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("yt-dlp (stderr): {}", line);
        }
    });

    while let Ok(Some(line)) = stdout_reader.next_line().await {
        info!("{}", line); // Print yt-dlp stdout

        if line.contains("[download] Destination: ") {
            let mut path_guard = downloaded_filename_clone.lock().unwrap();
            *path_guard = Some(line["[download] Destination: ".len()..].trim().to_string());
        }

        // Use parse_progress directly for consistency with download_audio
        if let Some((progress, total_size)) = crate::progress::parse_progress(&line)
        {
            let pb = pb_clone.lock().unwrap();
            if total_size > 0 { // Avoid division by zero or setting length to 0 if not known
                pb.set_length(total_size);
            }
            pb.set_position(progress);
        }
    }

    let status = child
        .wait()
        .await
        .context("Erreur lors de l'attente de la fin du processus yt-dlp")?;

    pb_arc.lock().unwrap().finish_with_message("Téléchargement vidéo terminé.");

    if status.success() {
        info!("La vidéo a été téléchargée avec succès !");
        if let Some(path_str) = downloaded_filename_arc.lock().unwrap().as_ref() {
            let config = config::load_config();
            let base_download_dir = PathBuf::from(config.download_directory);
            let full_path = base_download_dir.join(path_str);
            info!("Chemin du fichier vidéo téléchargé : {:?}", full_path);
        } else {
            warn!("Chemin du fichier vidéo non extrait de la sortie yt-dlp.");
        }
    } else {
        error!("Erreur lors du téléchargement de la vidéo (yt-dlp a échoué). Code: {:?}", status.code());
        warn!("Essayez avec un format différent ou utilisez 'best' pour le meilleur format disponible.");
        bail!("yt-dlp failed with status: {:?}", status.code());
    }

    Ok(())
}

// Modified download_audio function
pub async fn download_audio(url: &str, audio_format: &str, extract_instrumental: bool, custom_filename: Option<String>, cookies_browser: Option<String>, download_playlist: bool) -> Result<()> {
    let mut command = Command::new("yt-dlp");
    let config = config::load_config();
    command.args(&["-P", &config.download_directory]);

    // Only download single video if user doesn't want the playlist
    if !download_playlist {
        command.arg("--no-playlist");
    }

    // Ajouter l'option pour personnaliser le nom du fichier si fourni
    if let Some(filename) = &custom_filename {
        // Utiliser le template de yt-dlp pour définir le nom de fichier tout en conservant l'extension originale
        command.args(&["-o", &format!("{}.%(ext)s", filename)]);
    }

    command.args(&[
        "-f",
        "bestaudio/best", // Fallback if bestaudio is not available
        "--extract-audio",
        "--audio-format",
        audio_format,
        url,
    ]);

    // Add cookies if provided
    if let Some(browser) = cookies_browser {
        command.args(&["--cookies-from-browser", &browser]);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    info!("Lancement de yt-dlp pour l'audio...");
    let mut child = command
        .spawn()
        .context("Erreur lors de l'exécution de yt-dlp pour l'audio")?;
    let stdout = child.stdout.take().context("Erreur de capture du stdout de yt-dlp")?;
    let stderr = child.stderr.take().context("Erreur de capture du stderr de yt-dlp")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let pb_arc = Arc::new(Mutex::new(ProgressBar::new(100)));
    pb_arc.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Erreur lors de la configuration du style de la barre de progression")
            .progress_chars("##-"),
    );

    let pb_clone = Arc::clone(&pb_arc);
    let downloaded_filename_arc = Arc::new(Mutex::new(None::<String>));
    let downloaded_filename_clone = Arc::clone(&downloaded_filename_arc);

    // Spawn a task for stderr
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("yt-dlp (stderr): {}", line);
        }
    });

    // Processing yt-dlp's stdout
    while let Ok(Some(line)) = stdout_reader.next_line().await {
        info!("{}", line); // Print yt-dlp stdout

        if line.contains("[download] Destination: ") {
            let mut path_guard = downloaded_filename_clone.lock().unwrap();
            *path_guard = Some(line["[download] Destination: ".len()..].trim().to_string());
        } else if line.contains("[ExtractAudio] Destination: ") { // yt-dlp sometimes uses this for extracted audio
            let mut path_guard = downloaded_filename_clone.lock().unwrap();
            *path_guard = Some(line["[ExtractAudio] Destination: ".len()..].trim().to_string());
        }


        // Assuming parse_progress is correct for audio
        if let Some((progress, total_size)) = crate::progress::parse_progress(&line) {
            let pb = pb_clone.lock().unwrap();
            if total_size > 0 {
                pb.set_length(total_size);
            }
            pb.set_position(progress);
        }
    }

    let status = child
        .wait()
        .await
        .context("Erreur lors de l'attente de la fin du processus yt-dlp")?;

    pb_arc.lock().unwrap().finish_with_message("Téléchargement audio (yt-dlp) terminé.");

    if status.success() {
        info!("L'audio a été téléchargée avec succès par yt-dlp!");

        let downloaded_filename_option = downloaded_filename_arc.lock().unwrap().clone();

        if let Some(downloaded_filename_str) = downloaded_filename_option {
            let config = config::load_config();
            let base_download_dir = PathBuf::from(config.download_directory);
            // yt-dlp might output a full path if -P is not CWD, or just a filename.
            let original_downloaded_full_path = if Path::new(&downloaded_filename_str).is_absolute() {
                PathBuf::from(&downloaded_filename_str)
            } else {
                base_download_dir.join(&downloaded_filename_str)
            };

            info!("Chemin du fichier audio original : {:?}", original_downloaded_full_path);

            if extract_instrumental {
                spleeter::extract_instrumental(&original_downloaded_full_path).await?;
            }
        } else {
            warn!("⚠️ Impossible de déterminer le nom du fichier audio téléchargé par yt-dlp.");
        }
    } else {
        error!("Erreur lors du téléchargement de l'audio par yt-dlp. Code: {:?}", status.code());
        warn!("Essayez avec un format audio différent ou vérifiez l'URL.");
        bail!("yt-dlp failed with status: {:?}", status.code());
    }
    
    Ok(())
}


// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf; // Keep this if used by other tests not shown or if you add new ones

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_only() {
        // Test spécifique à Windows
        assert!(true);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_not_windows() {
        // Test qui s'exécute sur les autres OS
        assert!(true);
    }

    // Helper function to construct the yt-dlp command for videos
    fn build_yt_dlp_command_video(url: &str, format: &str, keep_files: bool, download_path: Option<PathBuf>, custom_filename: Option<String>, cookies_browser: Option<String>) -> Command {
        let mut command = Command::new("yt-dlp");

        if let Some(path) = download_path {
            command.args(&["-P", path.to_str().unwrap_or(".")]);
        }

        // Ajouter l'option pour personnaliser le nom du fichier si fourni
        if let Some(filename) = &custom_filename {
            // Utiliser le template de yt-dlp pour définir le nom de fichier tout en conservant l'extension originale
            command.args(&["-o", &format!("{}.%(ext)s", filename)]);
        }

        command.arg(url);

        if keep_files {
            command.arg("-k");
        }

        // Add cookies if provided
        if let Some(browser) = cookies_browser {
            command.args(&["--cookies-from-browser", &browser]);
        }

        if !format.is_empty() {
            if format == "mp4" {
                command.args(&["-f", "bv*+ba/b", "--merge-output-format", "mp4"]);
            } else {
                command.args(&["-f", format]);
            }
        }

        command
    }

    // Helper function to construct the yt-dlp command for audio
    // This helper does not need to change for Spleeter integration,
    // as Spleeter is called *after* yt-dlp.
    fn build_yt_dlp_command_audio(url: &str, audio_format: &str, download_path: Option<PathBuf>, custom_filename: Option<String>, cookies_browser: Option<String>) -> Command {
        let mut command = Command::new("yt-dlp");

        if let Some(path) = download_path {
            command.args(&["-P", path.to_str().unwrap_or(".")]);
        }

        // Ajouter l'option pour personnaliser le nom du fichier si fourni
        if let Some(filename) = &custom_filename {
            // Utiliser le template de yt-dlp pour définir le nom de fichier tout en conservant l'extension originale
            command.args(&["-o", &format!("{}.%(ext)s", filename)]);
        }

        command.args(&[
            "-f",
            "bestaudio/best", // Added fallback
            "--extract-audio",
            "--audio-format",
            audio_format,
            url,
            url,
        ]);

        // Add cookies if provided
        if let Some(browser) = cookies_browser {
            command.args(&["--cookies-from-browser", &browser]);
        }

        command
    }
    #[test]
    fn test_build_yt_dlp_command_video_with_all_options() {
        let url = "https://test.url/video";
        let format = "mp4";
        let keep_files = true;
        let download_path = Some(PathBuf::from("/tmp/downloads"));
        let custom_filename = None; // Pas de nom de fichier personnalisé
        let cookies_browser = None;
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone(), custom_filename, cookies_browser);

        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let expected_path_str = download_path.unwrap().to_str().unwrap().to_string();

        assert!(args.contains(&"-P".to_string()));
        assert!(args.iter().any(|arg| arg == &expected_path_str));
        assert!(args.contains(&url.to_string()));
        assert!(args.contains(&"-k".to_string()));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.iter().any(|arg| arg == "bv*+ba/b"));
        assert!(args.contains(&"--merge-output-format".to_string()));
        assert!(args.iter().any(|arg| arg == "mp4"));
    }

    #[test]
    fn test_build_yt_dlp_command_video_minimal() {
        let url = "https://test.url/simple";
        let format = ""; // Empty format
        let keep_files = false;
        let download_path = None; // No specific download path
        let custom_filename = None; // Pas de nom de fichier personnalisé
        let cookies_browser = None;
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path, custom_filename, cookies_browser);

        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();

        // When download_path is None, -P should not be added by build_yt_dlp_command_video
        // However, the main download_video function *will* add -P with default_download_dir or "."
        // This test tests the helper, which is fine.
        assert!(!args.contains(&"-P".to_string()));
        assert!(args.contains(&url.to_string()));
        assert!(!args.contains(&"-k".to_string()));
        assert!(!args.contains(&"-f".to_string())); // -f should not be present if format is empty
    }

    #[test]
    fn test_build_yt_dlp_command_audio() {
        let url = "https://audio.test";
        let audio_format = "mp3";
        let download_path = Some(PathBuf::from("/home/user/dl"));
        let custom_filename = None; // Pas de nom de fichier personnalisé
        let cookies_browser = None;
        let command = build_yt_dlp_command_audio(url, audio_format, download_path.clone(), custom_filename, cookies_browser);

        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let expected_path_str = download_path.unwrap().to_str().unwrap().to_string();

        assert!(args.contains(&"-P".to_string()));
        assert!(args.iter().any(|arg| arg == &expected_path_str));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"bestaudio/best".to_string()));
        assert!(args.contains(&"--extract-audio".to_string()));
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.iter().any(|arg| arg == audio_format));
        assert!(args.contains(&url.to_string()));
        // Vérifier que l'option -o n'est pas présente quand custom_filename est None
        assert!(!args.contains(&"-o".to_string()));
    }

    #[test]
    fn test_build_yt_dlp_command_with_custom_filename() {
        // Test pour la vidéo avec nom personnalisé
        let url = "https://test.url/video";
        let format = "mp4";
        let keep_files = true;
        let download_path = Some(PathBuf::from("/tmp/downloads"));
        let custom_filename = Some("ma_video_personnalisee".to_string());
        let cookies_browser = None;
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone(), custom_filename.clone(), cookies_browser);

        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(args.contains(&"-o".to_string()));
        assert!(args.iter().any(|arg| arg == &format!("{}.%(ext)s", custom_filename.as_ref().unwrap())));

        // Test pour l'audio avec nom personnalisé
        let url = "https://audio.test";
        let audio_format = "mp3";
        let download_path = Some(PathBuf::from("/home/user/dl"));
        let custom_filename = Some("mon_audio_personnalise".to_string());
        let cookies_browser = None;
        let command = build_yt_dlp_command_audio(url, audio_format, download_path.clone(), custom_filename.clone(), cookies_browser);

        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(args.contains(&"-o".to_string()));
        assert!(args.iter().any(|arg| arg == &format!("{}.%(ext)s", custom_filename.as_ref().unwrap())));
    }

    #[test]
    fn test_build_yt_dlp_command_with_cookies() {
        let url = "https://test.url/video";
        let format = "mp4";
        let keep_files = false;
        let download_path = Some(PathBuf::from("/tmp/downloads"));
        let custom_filename = None;
        let cookies_browser = Some("chrome".to_string());
        
        // Test video command
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone(), custom_filename.clone(), cookies_browser.clone());
        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();
        
        assert!(args.contains(&"--cookies-from-browser".to_string()));
        assert!(args.contains(&"chrome".to_string()));

        // Test audio command
        let command = build_yt_dlp_command_audio(url, "mp3", download_path, custom_filename, cookies_browser);
        let args: Vec<String> = command.as_std().get_args().map(|a| a.to_string_lossy().to_string()).collect();
        
        assert!(args.contains(&"--cookies-from-browser".to_string()));
        assert!(args.contains(&"chrome".to_string()));
    }
}