#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use colored::*;
use std::io::{self, Write};
use log::{info, warn, error};
use rmd_core::{commands, installers, cookies, downloader, settings};

mod user_input;

fn run_cli() {
    let spleeter_available = commands::check_command("spleeter");
    if !spleeter_available {
        warn!("{}", "Spleeter not found. Instrumental extraction will be disabled.".yellow());
    }

    if commands::check_command("curl") {
        info!("{}", "La commande 'curl' est disponible !".green());
    } else {
        warn!("{}", "La commande 'curl' n'est pas trouvée !".red());
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
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                info!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                downloader::download_video(&url, &format, keep_files, custom_filename);
            }
            "2" => {
                let url = demander_url();
                let (format, keep_files) = user_input::choisir_video_options_avances();
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                info!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                downloader::download_video(&url, &format, keep_files, custom_filename);
            }
            "3" => {
                let url = demander_url();
                let audio_format = user_input::choisir_audio_format();
                let _extract_instrumental = user_input::demander_extraction_instrumental(spleeter_available);
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                info!("{}", "\n📥 Téléchargement de l'audio en cours...\n".cyan().bold());
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

        if !user_input::demander_si_continuer() {
            info!(
                "{}",
                "\n👋 Merci d’avoir utilisé Panther Downloader. À bientôt !\n"
                    .blue()
                    .bold()
            );
            break;
        }
    }
}

fn afficher_interface(spleeter_available: bool) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║     🎶 Panther Downloader - Audio & Vidéo 🎶       ║");
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
            continue;
        }
        let url = url.trim();
        if url.is_empty() {
            warn!("{}", " L'URL ne peut pas être vide. Veuillez réessayer.".yellow());
        } else {
            return url.to_string();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--cli".to_string()) {
        run_cli();
    } else {
        tauri::Builder::default()
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
