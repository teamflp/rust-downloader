use dirs::download_dir;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::io::{BufRead, BufReader as StdBufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread; 

pub fn download_video(url: &str, format: &str, keep_files: bool) {
    let mut command = Command::new("yt-dlp");

    // Ajouter le répertoire de téléchargement par défaut
    if let Some(download_path) = download_dir() {
        command.args(&["-P", download_path.to_str().unwrap()]);
    }

    command.arg(url);

    if keep_files {
        command.arg("-k");
    }

    if !format.is_empty() {
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

    let pb = Arc::new(Mutex::new(ProgressBar::new(100)));
    pb.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Erreur lors de la configuration du style de la barre de progression")
            .progress_chars("##-"),
    );

    let pb_clone = Arc::clone(&pb);
    let path = Arc::new(Mutex::new(None));
    let path_clone = Arc::clone(&path);

    thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });

    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            println!("{}", line);

            if line.contains("[download] Destination: ") {
                let mut path = path_clone.lock().unwrap();
                *path = Some(line["[download] Destination: ".len()..].to_string());
            }

            if let Some((progress, total_size)) = crate::progress::afficher_progression_ligne(&line)
            {
                let pb = pb_clone.lock().unwrap();
                pb.set_length(total_size);
                pb.set_position(progress);
            }
        }
    }

    let status = child
        .wait()
        .expect("Erreur lors de l'attente de la fin du processus");

    pb.lock().unwrap().finish();

    if status.success() {
        println!("La vidéo a été téléchargée avec succès !");
        if let Some(path) = path.lock().unwrap().as_ref() {
            // Utiliser le répertoire de téléchargement par défaut
            let download_dir = download_dir().unwrap_or_else(|| {
                env::current_dir().expect("Impossible d'obtenir le répertoire de travail actuel")
            });
            let full_path = download_dir.join(path);
            println!("Chemin du fichier téléchargé : {:?}", full_path);
        }
    } else {
        eprintln!("Erreur lors du téléchargement de la vidéo.");
        std::process::exit(1);
    }
}

pub fn download_audio(url: &str, audio_format: &str) {
    let mut command = Command::new("yt-dlp");

    // Ajouter le répertoire de téléchargement par défaut
    if let Some(download_path) = download_dir() {
        command.args(&["-P", download_path.to_str().unwrap()]);
    }

    command.args(&[
        "-f",
        "bestaudio",
        "--extract-audio",
        "--audio-format",
        audio_format,
        url,
    ]);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .expect("Erreur lors de l'exécution de yt-dlp");
    let stdout = child.stdout.take().expect("Erreur de capture du stdout");
    let stderr = child.stderr.take().expect("Erreur de capture du stderr");

    let stdout_reader = StdBufReader::new(stdout);
    let stderr_reader = StdBufReader::new(stderr);

    let pb = Arc::new(Mutex::new(ProgressBar::new(100)));
    pb.lock().unwrap().set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("Erreur lors de la configuration du style de la barre de progression")
            .progress_chars("##-"),
    );

    let pb_clone = Arc::clone(&pb);
    let path = Arc::new(Mutex::new(None));
    let path_clone = Arc::clone(&path);

    thread::spawn(move || {
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });

    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            println!("{}", line);

            if line.contains("[download] Destination: ") {
                let mut path = path_clone.lock().unwrap();
                *path = Some(line["[download] Destination: ".len()..].to_string());
            }

            if let Some((progress, total_size)) = crate::progress::parse_progress(&line) {
                let pb = pb_clone.lock().unwrap();
                pb.set_length(total_size);
                pb.set_position(progress);
            }
        }
    }

    let status = child
        .wait()
        .expect("Erreur lors de l'attente de la fin du processus");

    pb.lock().unwrap().finish();

    if status.success() {
        println!("L'audio a été téléchargée avec succès !");
        if let Some(path) = path.lock().unwrap().as_ref() {
            // Utiliser le répertoire de téléchargement par défaut
            let download_dir = download_dir().unwrap_or_else(|| {
                env::current_dir().expect("Impossible d'obtenir le répertoire de travail actuel")
            });
            let full_path = download_dir.join(path);
            println!("Chemin du fichier téléchargé : {:?}", full_path);
        }
    } else {
        eprintln!("Erreur lors du téléchargement de l'audio.");
        std::process::exit(1);
    }
}

// TESTS 
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
    fn build_yt_dlp_command_video(url: &str, format: &str, keep_files: bool, download_path: Option<PathBuf>, ) -> Command {
        let mut command = Command::new("yt-dlp");

        if let Some(path) = download_path {
            command.args(&["-P", path.to_str().unwrap()]);
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
    fn build_yt_dlp_command_audio(url: &str, audio_format: &str, download_path: Option<PathBuf>, ) -> Command {
        let mut command = Command::new("yt-dlp");

        if let Some(path) = download_path {
            command.args(&["-P", path.to_str().unwrap()]);
        }

        command.args(&[
            "-f",
            "bestaudio",
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
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path.clone());

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let expected_path = download_path.unwrap().to_str().unwrap().to_string();

        assert!(args.contains(&"-P".to_string()));
        assert!(args.contains(&expected_path));
        assert!(args.contains(&url.to_string()));
        assert!(args.contains(&"-k".to_string()));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&format.to_string()));
    }

    #[test]
    fn test_build_yt_dlp_command_video_minimal() {
        let url = "https://test.url/simple";
        let format = "";
        let keep_files = false;
        let download_path = None;
        let command = build_yt_dlp_command_video(url, format, keep_files, download_path);

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();

        assert!(!args.contains(&"-P".to_string()));
        assert!(args.contains(&url.to_string()));
        assert!(!args.contains(&"-k".to_string()));
        assert!(!args.contains(&"-f".to_string()));
    }

    #[test]
    fn test_build_yt_dlp_command_audio() {
        let url = "https://audio.test";
        let audio_format = "mp3";
        let download_path = Some(PathBuf::from("/home/user/dl"));
        let command = build_yt_dlp_command_audio(url, audio_format, download_path.clone());

        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        let expected_path = download_path.unwrap().to_str().unwrap().to_string();

        assert!(args.contains(&"-P".to_string()));
        assert!(args.contains(&expected_path));
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"bestaudio".to_string()));
        assert!(args.contains(&"--extract-audio".to_string()));
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.contains(&audio_format.to_string()));
        assert!(args.contains(&url.to_string()));
    }
}