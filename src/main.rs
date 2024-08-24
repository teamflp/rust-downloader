mod downloader; // Déclare le module downloader
mod commands;
mod installers;
mod user_input;
mod progress;

use std::io;

fn main() {
    // Vérifier si ffmpeg est installé
    if !commands::check_command("ffmpeg") {
        installers::install_ffmpeg();
    } else {
        println!("ffmpeg est déjà installé.");
    }

    // Vérifier si yt-dlp est installé
    if !commands::check_command("yt-dlp") {
        installers::install_yt_dlp();
    } else {
        println!("yt-dlp est déjà installé.");
    }

    loop {
        // Demander à l'utilisateur l'URL de la vidéo à télécharger
        println!("Entrez l'URL de la vidéo à télécharger :");
        let mut url = String::new();
        io::stdin().read_line(&mut url).expect("Erreur de lecture de l'URL");
        let url = url.trim();

        // Demander à l'utilisateur de choisir le format de téléchargement
        println!("Choisissez le format de téléchargement (vidéo/audio) :");
        let mut format_choice = String::new();
        io::stdin().read_line(&mut format_choice).expect("Erreur de lecture du choix de format");
        let format_choice = format_choice.trim().to_lowercase();

        match format_choice.as_str() {
            "vidéo" | "video" => {
                let (format, keep_files) = user_input::choisir_format_et_options();
                downloader::download_video(url, &format, keep_files);
            },
            "audio" => {
                let audio_format = user_input::choisir_audio_format();
                downloader::download_audio(url, &audio_format);
            },
            _ => {
                eprintln!("Format non reconnu. Veuillez choisir 'vidéo' ou 'audio'.");
                continue; // Demander de nouveau le format si l'entrée est incorrecte
            }
        }

        // Demander à l'utilisateur s'il souhaite continuer
        if !user_input::demander_si_continuer() {
            println!("Merci d'avoir utilisé le programme de téléchargement !");
            break; // Quitter la boucle et terminer le programme
        }
    }
}
