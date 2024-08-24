use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader as StdBufReader};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;
use dirs::download_dir; // Importer la fonction pour obtenir le répertoire de téléchargement

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

    let mut child = command.spawn().expect("Erreur lors de l'exécution de yt-dlp");
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

    let status = child.wait().expect("Erreur lors de l'attente de la fin du processus");

    pb.lock().unwrap().finish();

    if status.success() {
        println!("La vidéo a été téléchargée avec succès !");
        if let Some(path) = path.lock().unwrap().as_ref() {
            // Utiliser le répertoire de téléchargement par défaut
            let download_dir = download_dir().unwrap_or_else(|| env::current_dir().expect("Impossible d'obtenir le répertoire de travail actuel"));
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

    command.args(&["-f", "bestaudio", "--extract-audio", "--audio-format", audio_format, url]);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().expect("Erreur lors de l'exécution de yt-dlp");
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

    let status = child.wait().expect("Erreur lors de l'attente de la fin du processus");

    pb.lock().unwrap().finish();

    if status.success() {
        println!("L'audio a été téléchargée avec succès !");
        if let Some(path) = path.lock().unwrap().as_ref() {
            // Utiliser le répertoire de téléchargement par défaut
            let download_dir = download_dir().unwrap_or_else(|| env::current_dir().expect("Impossible d'obtenir le répertoire de travail actuel"));
            let full_path = download_dir.join(path);
            println!("Chemin du fichier téléchargé : {:?}", full_path);
        }
    } else {
        eprintln!("Erreur lors du téléchargement de l'audio.");
        std::process::exit(1);
    }
}
