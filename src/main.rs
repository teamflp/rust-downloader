use crate::commands::check_command;
use colored::*;
use installers::ensure_dependencies;
use std::io::{self, Write};
use log::{info, warn, error};

mod commands;
mod downloader;
mod installers;
mod progress;
mod user_input;
mod commands_test;
#[cfg(test)]
mod cookies_test;
#[cfg(test)]
mod config_test;
mod cookies;
mod config;
mod settings;

fn main() {
    env_logger::init();
    // 🛠️ Vérification de la présence de yt-dlp et ffmpeg
    ensure_dependencies();

    let spleeter_available = check_command("spleeter");
    if !spleeter_available {
        warn!("{}", "Spleeter not found. Instrumental extraction will be disabled.".yellow());
    }

    // 💡 Vérification de la présence de "curl" (à adapter si besoin)
    if check_command("curl") {
        info!("{}", "La commande 'curl' est disponible !".green());
    } else {
        warn!("{}", "La commande 'curl' n'est pas trouvée !".red());
        // std::process::exit(1); // Décommentez si curl est indispensable
    }

    loop {
        afficher_interface(spleeter_available);

        print!("{}", "👉 Votre choix : ".bold());
        io::stdout().flush().unwrap_or_else(|e| {
            error!("Erreur lors du flush stdout: {}", e);
        });

        let mut choix = String::new();
        if io::stdin().read_line(&mut choix).is_err() {
            error!("{}", "❌ Erreur de lecture de votre choix.".red());
            continue;
        }
        let choix = choix.trim();

        if choix.eq_ignore_ascii_case("q") {
            info!(
                "{}",
                "\n👋 Merci d’avoir utilisé Panther Downloader. À bientôt !\n"
                    .blue()
                    .bold()
            );
            break;
        }

        match choix {
            "1" => {
                let url = demander_url();
                let (format, keep_files) = user_input::choisir_format_et_options();
                // Demander si l'utilisateur souhaite personnaliser le nom du fichier
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                info!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                // Note: La fonction download_video n'est pas modifiée pour l'extraction instrumentale
                downloader::download_video(&url, &format, keep_files, custom_filename);
            }
            "2" => {
                let url = demander_url();
                let (format, keep_files) = user_input::choisir_video_options_avances();
                // Demander si l'utilisateur souhaite personnaliser le nom du fichier
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                info!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                // Note: La fonction download_video n'est pas modifiée pour l'extraction instrumentale
                downloader::download_video(&url, &format, keep_files, custom_filename);
            }
            "3" => {
                let url = demander_url();
                let audio_format = user_input::choisir_audio_format();
                // On demande ici si l'utilisateur veut l'instrumental
                let _extract_instrumental = user_input::demander_extraction_instrumental(spleeter_available);
                // Demander si l'utilisateur souhaite personnaliser le nom du fichier
                let custom_filename = user_input::demander_nom_fichier_personnalise();

                info!("{}", "\n📥 Téléchargement de l'audio en cours...\n".cyan().bold());
                // Assurez-vous que votre fonction download_audio accepte ce nouveau paramètre
                downloader::download_audio(&url, &audio_format, _extract_instrumental, custom_filename);
            }
            "4" => {
                let url = demander_url();
                cookies::extract_cookies_and_download(&url);
            }
            "5" => {
                settings::show_settings_menu();
            }
            _ => {
                warn!("{}", "❌ Choix invalide. Veuillez entrer 1, 2, 3, 4, 5 ou q.".red());
                continue;
            }
        }

        // Utilise la fonction centralisée pour demander si on continue
        if !user_input::demander_si_continuer() {
            info!(
                "{}",
                "\n👋 Merci d’avoir utilisé Rust Media Downloader. À bientôt !\n"
                    .blue()
                    .bold()
            );
            break;
        }
    }
}

fn afficher_interface(spleeter_available: bool) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║     🎶 Rust Media Downloader - Audio & Vidéo 🎶    ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    println!("{}", "--- Downloads ---".bold());
    println!("   [1] 🎥 Download Video (Quick)");
    println!("   [2] 🎬 Download Video (Advanced)");
    if spleeter_available {
        println!("   [3] 🎧 Download Audio (with instrumental extraction)");
    } else {
        println!("   [3] 🎧 Download Audio {}", "(instrumental extraction disabled)".dimmed());
    }
    println!("   [4] 🍪 Download with Cookies");
    println!("");
    println!("{}", "--- Management ---".bold());
    println!("   [5] ⚙️  Settings");
    println!("   [q] ❌ Quit");
}

fn demander_url() -> String {
    loop {
        print!("{}", "🔗 Entrez l'URL (ex: YouTube, Soundcloud) :\n👉 ".bold());
        io::stdout().flush().unwrap_or_else(|e| {
            error!("Erreur lors du flush stdout: {}", e);
        });

        let mut url = String::new();
        if io::stdin().read_line(&mut url).is_err() {
            error!("{}", "❌ Erreur de lecture de l'URL.".red());
            continue; // Redemande l'URL en cas d'erreur de lecture
        }
        let url = url.trim();
        if url.is_empty() {
            warn!("{}", " L'URL ne peut pas être vide. Veuillez réessayer.".yellow());
        } else {
            return url.to_string();
        }
    }
}