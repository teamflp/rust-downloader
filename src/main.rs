use crate::commands::check_command;
use colored::*;
use installers::ensure_dependencies;
use std::io::{self, Write};

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
    // 🛠️ Vérification de la présence de yt-dlp et ffmpeg
    ensure_dependencies();

    // 💡 Vérification de la présence de "curl" (à adapter si besoin)
    if check_command("curl") {
        println!("{}", "La commande 'curl' est disponible !".green());
    } else {
        println!("{}", "La commande 'curl' n'est pas trouvée !".red());
        // std::process::exit(1); // Décommentez si curl est indispensable
    }

    loop {
        afficher_interface();

        print!("{}", "👉 Votre choix : ".bold());
        io::stdout().flush().unwrap_or_else(|e| {
            eprintln!("Erreur lors du flush stdout: {}", e);
        });

        let mut choix = String::new();
        if io::stdin().read_line(&mut choix).is_err() {
            println!("{}", "❌ Erreur de lecture de votre choix.".red());
            continue;
        }
        let choix = choix.trim();

        if choix.eq_ignore_ascii_case("q") {
            println!(
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
                println!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                // Note: La fonction download_video n'est pas modifiée pour l'extraction instrumentale
                downloader::download_video(&url, &format, keep_files, custom_filename);
            }
            "2" => {
                let url = demander_url();
                let audio_format = user_input::choisir_audio_format();
                // On demande ici si l'utilisateur veut l'instrumental
                let _extract_instrumental = user_input::demander_extraction_instrumental();
                // Demander si l'utilisateur souhaite personnaliser le nom du fichier
                let custom_filename = user_input::demander_nom_fichier_personnalise();

                println!("{}", "\n📥 Téléchargement de l'audio en cours...\n".cyan().bold());
                // Assurez-vous que votre fonction download_audio accepte ce nouveau paramètre
                downloader::download_audio(&url, &audio_format, _extract_instrumental, custom_filename);
            }
            "3" => {
                let url = demander_url();
                cookies::extract_cookies_and_download(&url);
            }
            "4" => {
                settings::show_settings_menu();
            }
            _ => {
                println!("{}", "❌ Choix invalide. Veuillez entrer 1, 2, 3, 4 ou q.".red());

                continue;
            }
        }

        // Utilise la fonction centralisée pour demander si on continue
        if !user_input::demander_si_continuer() {
            println!(
                "{}",
                "\n👋 Merci d’avoir utilisé Panther Downloader. À bientôt !\n"
                    .blue()
                    .bold()
            );
            break;
        }
    }
}

fn afficher_interface() {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║     🎶 Panther Downloader - Audio & Vidéo 🎶       ║"); 
    println!("╚══════════════════════════════════════════════════╝\n");

    println!("Choisissez une option :");
    println!("   [1] 🎥 Télécharger une vidéo");
    println!("   [2] 🎧 Télécharger de l'audio (avec option instrumental)");
    println!("   [3] 🍪 Télécharger avec les cookies du navigateur (pour les vidéos privées)");
    println!("   [4] ⚙️  Settings");
    println!("   [q] ❌ Quitter");
}

fn demander_url() -> String {
    loop {
        print!("{}", "🔗 Entrez l'URL (ex: YouTube, Soundcloud) :\n👉 ".bold());
        io::stdout().flush().unwrap_or_else(|e| {
            eprintln!("Erreur lors du flush stdout: {}", e);
        });

        let mut url = String::new();
        if io::stdin().read_line(&mut url).is_err() {
            println!("{}", "❌ Erreur de lecture de l'URL.".red());
            continue; // Redemande l'URL en cas d'erreur de lecture
        }
        let url = url.trim();
        if url.is_empty() {
            println!("{}", " L'URL ne peut pas être vide. Veuillez réessayer.".yellow());
        } else {
            return url.to_string();
        }
    }
}
