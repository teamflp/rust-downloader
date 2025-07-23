use crate::config;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader as StdBufReader};
use log::{info, warn, error};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs; // Added for file.md operations
use std::path::{Path, PathBuf}; // Added for path manipulation

pub fn download_video(url: &str, format: &str, keep_files: bool, custom_filename: Option<String>) {
    let mut command = Command::new("yt-dlp");

    let config = config::load_config();
    command.args(&["-P", &config.download_directory]);

    // Ajouter l'option pour personnaliser le nom du fichier si fourni
    if let Some(filename) = &custom_filename {
        // Utiliser le template de yt-dlp pour définir le nom de fichier tout en conservant l'extension originale
        command.args(&["-o", &format!("{}.%(ext)s", filename)]);
    }

    command.arg(url);

    if keep_files {
        command.arg("-k");
    }

    if !format.is_empty() && format != "best" {
        command.args(&["-f", format]);
    }

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .expect("Erreur lors de l'exécution de yt-dlp");
    let stdout = child.stdout.take().expect("Erreur de capture du stdout");
    let stderr = child.stderr.take().expect("Erreur de capture du stderr");

    let stdout_reader = StdBufReader::new(stdout);
    let stderr_reader = StdBufReader::new(stderr);

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

    thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                // Consider printing stderr to stderr stream
                error!("yt-dlp (stderr): {}", line);
            }
        }
    });

    for line in stdout_reader.lines() {
        if let Ok(line) = line {
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
    }

    let status = child
        .wait()
        .expect("Erreur lors de l'attente de la fin du processus yt-dlp");

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
        // Removed exit(1) to prevent application from stopping abruptly
        warn!("Essayez avec un format différent ou utilisez 'best' pour le meilleur format disponible.");
    }
}

// Modified download_audio function
pub fn download_audio(url: &str, audio_format: &str, extract_instrumental: bool, custom_filename: Option<String>) {
    let mut command = Command::new("yt-dlp");
    let config = config::load_config();
    command.args(&["-P", &config.download_directory]);

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

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    info!("Lancement de yt-dlp pour l'audio...");
    let mut child = command
        .spawn()
        .expect("Erreur lors de l'exécution de yt-dlp pour l'audio");
    let stdout = child.stdout.take().expect("Erreur de capture du stdout de yt-dlp");
    let stderr = child.stderr.take().expect("Erreur de capture du stderr de yt-dlp");

    let stdout_reader = StdBufReader::new(stdout);
    let stderr_reader = StdBufReader::new(stderr);

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

    // Thread for yt-dlp's stderr
    thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                error!("yt-dlp (stderr): {}", line);
            }
        }
    });

    // Processing yt-dlp's stdout
    for line in stdout_reader.lines() {
        if let Ok(line) = line {
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
    }

    let status = child
        .wait()
        .expect("Erreur lors de l'attente de la fin du processus yt-dlp");

    pb_arc.lock().unwrap().finish_with_message("Téléchargement audio (yt-dlp) terminé.");

    if status.success() {
        info!("L'audio a été téléchargée avec succès par yt-dlp!");

        let downloaded_filename_option = downloaded_filename_arc.lock().unwrap().clone();

        if let Some(downloaded_filename_str) = downloaded_filename_option {
            let config = config::load_config();
            let base_download_dir = PathBuf::from(config.download_directory);
            // yt-dlp might output a full path if -P is not CWD, or just a filename.
            // The extracted filename_str might already be a full path in some yt-dlp versions/configs.
            // Let's assume filename_str is the final filename *relative to base_download_dir* or an absolute path.
            let original_downloaded_full_path = if Path::new(&downloaded_filename_str).is_absolute() {
                PathBuf::from(&downloaded_filename_str)
            } else {
                base_download_dir.join(&downloaded_filename_str)
            };


            info!("Chemin du fichier audio original : {:?}", original_downloaded_full_path);

            if extract_instrumental {
                info!("⚙️  Extraction de l'instrumental avec Spleeter en cours (cela peut prendre du temps)...");

                if Command::new("spleeter").arg("--version").output().is_err() {
                    error!("❌ Spleeter n'est pas installé ou n'est pas dans le PATH.");
                    info!("   Veuillez l'installer pour utiliser l'extraction instrumentale.");
                    info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
                    // Not exiting, user gets the original audio.
                    return;
                }

                let input_audio_path_for_spleeter = match original_downloaded_full_path.to_str() {
                    Some(path) => path,
                    None => {
                        error!("❌ Chemin du fichier audio original invalide pour Spleeter.");
                        return;
                    }
                };

                let spleeter_output_parent_dir = original_downloaded_full_path.parent().unwrap_or_else(|| Path::new("."));

                info!("Spleeter utilisera le dossier de sortie : {:?}", spleeter_output_parent_dir);

                let spinner_style = ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap();
                let pb = ProgressBar::new_spinner();
                pb.set_style(spinner_style);
                pb.set_message("Spleeter is working...");

                let spleeter_cmd = Command::new("spleeter")
                    .arg("separate")
                    .arg("-p")
                    .arg("spleeter:2stems") // Separates into vocals and accompaniment
                    .arg("-o")
                    .arg(spleeter_output_parent_dir.to_str().unwrap()) // Spleeter creates a subfolder here
                    .arg(input_audio_path_for_spleeter)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();

                match spleeter_cmd {
                    Ok(mut spleeter_child) => {
                        info!("Spleeter démarré...");
                        // Capture Spleeter's output (optional, can be verbose)
                        if let Some(s_stdout) = spleeter_child.stdout.take() {
                            let reader = StdBufReader::new(s_stdout);
                            for line in reader.lines().filter_map(Result::ok) {
                                info!("Spleeter (stdout): {}", line);
                            }
                        }
                        if let Some(s_stderr) = spleeter_child.stderr.take() {
                            let reader = StdBufReader::new(s_stderr);
                            for line in reader.lines().filter_map(Result::ok) {
                                error!("Spleeter (stderr): {}", line);
                            }
                        }

                        let spleeter_status = spleeter_child.wait().expect("Spleeter a échoué lors de l'attente.");
                        pb.finish_with_message("Spleeter finished.");

                        if spleeter_status.success() {
                            info!("✅ Spleeter a terminé l'extraction.");

                            let original_file_stem = original_downloaded_full_path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("audio_file");

                            // Spleeter creates a directory named after the input file.md's stem
                            let spleeter_output_subdir = spleeter_output_parent_dir.join(original_file_stem);
                            let instrumental_spleeter_filename = "accompaniment.wav"; // Default for 2stems
                            let vocals_spleeter_filename = "vocals.wav";

                            let spleeter_instrumental_path = spleeter_output_subdir.join(instrumental_spleeter_filename);
                            let spleeter_vocals_path = spleeter_output_subdir.join(vocals_spleeter_filename);

                            if spleeter_instrumental_path.exists() {
                                let final_instrumental_filename = format!("{}_instrumental.wav", original_file_stem);
                                let final_instrumental_full_path = original_downloaded_full_path.with_file_name(final_instrumental_filename);

                                match fs::rename(&spleeter_instrumental_path, &final_instrumental_full_path) {
                                    Ok(_) => {
                                        info!("🎶 Fichier instrumental sauvegardé ici : {:?}", final_instrumental_full_path);
                                        // Cleanup
                                        if let Err(e) = fs::remove_file(&original_downloaded_full_path) {
                                            warn!("⚠️ Impossible de supprimer le fichier audio original complet {:?}: {}", original_downloaded_full_path, e);
                                        }
                                        if spleeter_vocals_path.exists() {
                                            if let Err(e) = fs::remove_file(&spleeter_vocals_path) {
                                                warn!("⚠️ Impossible de supprimer le fichier vocal {:?}: {}", spleeter_vocals_path, e);
                                            }
                                        }
                                        if spleeter_output_subdir.exists() {
                                            if let Err(e) = fs::remove_dir_all(&spleeter_output_subdir) {
                                                warn!("⚠️ Impossible de supprimer le dossier de Spleeter {:?}: {}", spleeter_output_subdir, e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("❌ Erreur lors du renommage/déplacement du fichier instrumental: {}", e);
                                        info!("   L'instrumental brut de Spleeter se trouve peut-être ici : {:?}", spleeter_instrumental_path);
                                        info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
                                    }
                                }
                            } else {
                                error!("❌ Fichier instrumental ('{}') non trouvé dans le dossier de sortie de Spleeter: {:?}", instrumental_spleeter_filename, spleeter_output_subdir);
                                info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
                            }
                        } else {
                            error!("❌ Spleeter a échoué avec le code de sortie : {:?}. Le fichier audio original a été conservé.", spleeter_status.code());
                            info!("   Chemin du fichier original : {:?}", original_downloaded_full_path);
                        }
                    }
                    Err(e) => {
                        error!("❌ Erreur lors du lancement de Spleeter: {}. Le fichier audio original a été conservé.", e);
                        info!("   Chemin du fichier original : {:?}", original_downloaded_full_path);
                    }
                }
            }
            // If not extracting instrumental, the original audio is already there and its path printed.
        } else {
            warn!("⚠️ Impossible de déterminer le nom du fichier audio téléchargé par yt-dlp.");
        }
    } else {
        error!("Erreur lors du téléchargement de l'audio par yt-dlp. Code: {:?}", status.code());
        // Removed exit(1) to prevent application from stopping abruptly
        warn!("Essayez avec un format audio différent ou vérifiez l'URL.");
    }
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
    fn build_yt_dlp_command_video(url: &str, format: &str, keep_files: bool, download_path: Option<PathBuf>, custom_filename: Option<String>) -> Command {
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

        if !format.is_empty() {
            command.args(&["-f", format]);
        }

        command
    }

    // Helper function to construct the yt-dlp command for audio
    // This helper does not need to change for Spleeter integration,
    // as Spleeter is called *after* yt-dlp.
    fn build_yt_dlp_command_audio(url: &str, audio_format: &str, download_path: Option<PathBuf>, custom_filename: Option<String>) -> Command {
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
        ]);

        command
    }
    #[test]
    fn test_build_yt_dlp_command_video_with_all_options() {
        let url = "https://test.url/video";
        let format = "mp4";
        let keep_files = true;
        let download_path = Some(PathBuf::from("/tmp/downloads"));
        let custom_filename = None; // Pas de nom de fichier personnalisé
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone(), custom_filename);

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let expected_path_str = download_path.unwrap().to_str().unwrap().to_string();

        assert!(args.contains(&"-P".to_string()));
        assert!(args.iter().any(|arg| arg == &expected_path_str));
        assert!(args.contains(&url.to_string()));
        assert!(args.contains(&"-k".to_string()));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.iter().any(|arg| arg == format));
    }

    #[test]
    fn test_build_yt_dlp_command_video_minimal() {
        let url = "https://test.url/simple";
        let format = ""; // Empty format
        let keep_files = false;
        let download_path = None; // No specific download path
        let custom_filename = None; // Pas de nom de fichier personnalisé
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path, custom_filename);

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();

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
        let command = build_yt_dlp_command_audio(url, audio_format, download_path.clone(), custom_filename);

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
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
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone(), custom_filename.clone());

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(args.contains(&"-o".to_string()));
        assert!(args.iter().any(|arg| arg == &format!("{}.%(ext)s", custom_filename.as_ref().unwrap())));

        // Test pour l'audio avec nom personnalisé
        let url = "https://audio.test";
        let audio_format = "mp3";
        let download_path = Some(PathBuf::from("/home/user/dl"));
        let custom_filename = Some("mon_audio_personnalise".to_string());
        let command = build_yt_dlp_command_audio(url, audio_format, download_path.clone(), custom_filename.clone());

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(args.contains(&"-o".to_string()));
        assert!(args.iter().any(|arg| arg == &format!("{}.%(ext)s", custom_filename.as_ref().unwrap())));
    }
}