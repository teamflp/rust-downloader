use crate::commands::check_command;
use colored::*;
use installers::ensure_dependencies;
use std::io::{self, Write};

mod commands;
mod downloader;
mod installers;
mod progress;
mod user_input;

fn main() {
    // 🛠️ Vérification de la présence de yt-dlp et ffmpeg
    ensure_dependencies();

    // 💡 Vérification de la présence de "curl" (à adapter si besoin)
    if check_command("curl") {
        println!("{}", "La commande 'curl' est disponible !".green());
    } else {
        println!("{}", "La commande 'curl' n'est pas trouvée !".red());
        // std::process::exit(1);
    }


    loop {
        afficher_interface();

        print!("{}", "👉 Votre choix : ".bold());
        io::stdout().flush().unwrap();

        let mut choix = String::new();
        io::stdin().read_line(&mut choix).unwrap();
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
                println!("{}", "\n📥 Téléchargement en cours...\n".cyan().bold());
                downloader::download_video(&url, &format, keep_files);
            }
            "2" => {
                let url = demander_url();
                let format = user_input::choisir_audio_format();
                println!("{}", "\n📥 Téléchargement en cours...\n".cyan().bold());
                downloader::download_audio(&url, &format);
            }
            _ => {
                println!("{}", "❌ Choix invalide. Veuillez entrer 1 ou 2.".red());
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
    println!("║     🎬 Téléchargement de contenu vidéo et audio   ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    println!("1. Choisissez le type de téléchargement :");
    println!("   [1] 🎥 Vidéo");
    println!("   [2] 🎧 Audio");
    println!("   [q] ❌ Quitter");
}

fn demander_url() -> String {
    print!("{}", "Entrez l'URL YouTube :\n👉 ".bold());
    io::stdout().flush().unwrap();

    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    url.trim().to_string()
}
