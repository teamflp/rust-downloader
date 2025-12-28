use clap::Parser;
use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use log::{info, warn, error};

// Use shared library
use rust_media_downloader_shared::{
    download_video, download_audio,
    check_command, ensure_dependencies,
    config, cookies,
};

// Keep local modules for CLI-specific functionality
mod user_input;
mod settings;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// URL of the media to download
    #[arg(required = false)]
    url: Option<String>,

    /// Format (mp3, mp4, etc.)
    #[arg(short, long, default_value = "best")]
    format: String,

    /// Audio only
    #[arg(short, long)]
    audio: bool,

    /// Extract instrumental (requires Spleeter)
    #[arg(short, long)]
    instrumental: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();

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
    }

    // CLI Mode
    if let Some(url) = cli.url {
        if cli.audio {
             download_audio(&url, &cli.format, cli.instrumental, None, None, false).await?;
        } else {
             download_video(&url, &cli.format, false, None, None, false).await?;
        }
        return Ok(());
    }

    // Interactive Mode
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
                let cookies = user_input::demander_cookies();

                println!("{}", "\n═══════════════════════════════════════════════════════════".bright_blue());
                info!("{}", "\nTéléchargement de la vidéo en cours...\n".cyan().bold());
                if let Err(e) = download_video(&url, &format, keep_files, custom_filename, cookies, false).await {
                    error!("Erreur lors du téléchargement: {}", e);
                }
            }
            "2" => {
                let url = demander_url();
                let (format, keep_files) = user_input::choisir_video_options_avances();
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                let cookies = user_input::demander_cookies();

                println!("{}", "\n═══════════════════════════════════════════════════════════".bright_blue());
                info!("{}", "\n📥 Téléchargement de la vidéo en cours...\n".cyan().bold());
                if let Err(e) = download_video(&url, &format, keep_files, custom_filename, cookies, false).await {
                    error!("Erreur lors du téléchargement: {}", e);
                }
            }
            "3" => {
                let url = demander_url();
                let audio_format = user_input::choisir_audio_format();
                let _extract_instrumental = user_input::demander_extraction_instrumental(spleeter_available);
                let custom_filename = user_input::demander_nom_fichier_personnalise();
                let cookies = user_input::demander_cookies();

                println!("{}", "\n═══════════════════════════════════════════════════════════".bright_blue());
                info!("{}", "\n🎵 Téléchargement de l'audio en cours...\n".cyan().bold());
                if let Err(e) = download_audio(&url, &audio_format, _extract_instrumental, custom_filename, cookies, false).await {
                    error!("Erreur lors du téléchargement: {}", e);
                }
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
            println!("{}", "\n═══════════════════════════════════════════════════════════".bright_magenta());
            info!(
                "{}",
                "\n👋 Merci d'avoir utilisé Rust Media Downloader. À bientôt !\n"
                    .bright_magenta()
                    .bold()
            );
            println!("{}", "═══════════════════════════════════════════════════════════\n".bright_magenta());
            break;
        }
    }
    Ok(())
}

fn afficher_interface(spleeter_available: bool) {
    // Clear screen for better visual experience
    print!("\x1B[2J\x1B[1;1H");
    
    // Fancy top border
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan().bold());
    println!("{}", "║                                                           ║".bright_cyan().bold());
    println!(
        "{} {} {}",
"║".bright_cyan().bold(),
        "         Rust Media Downloader - Audio & Vidéo           ".bright_magenta().bold(),
        "║".bright_cyan().bold()
    );
    println!("{}", "║                                                           ║".bright_cyan().bold());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan().bold());
    println!("");
    
    // Downloads section with improved styling
    println!("{}", "┌───────────────────────────────────────────────────────────┐".cyan());
    println!(
        "{} {} {}",
        "│".cyan(),
        "  DOWNLOADS".bright_yellow().bold(),
        "                                              │".cyan()
    );
    println!("{}", "├───────────────────────────────────────────────────────────┤".cyan());
    println!(
        "{} {}",
        "│".cyan(),
        "  [1] 🎥  Download Video (Quick)                          │".bright_white()
    );
    println!(
        "{} {}",
        "│".cyan(),
        "  [2] 🎬  Download Video (Advanced)                       │".bright_white()
    );
    
    if spleeter_available {
        println!(
            "{} {}",
            "│".cyan(),
            "  [3] 🎧  Download Audio (with instrumental)            │".bright_white()
        );
    } else {
        println!(
            "{} {} {}",
            "│".cyan(),
            "  [3] 🎧  Download Audio".bright_white(),
            "(instrumental disabled)          │".dimmed()
        );
    }
    
    println!(
        "{} {}",
        "│".cyan(),
        "  [4] 🍪  Download with Cookies                           │".bright_white()
    );
    println!("{}", "└───────────────────────────────────────────────────────────┘".cyan());
    println!("");
    
    // Management section
    println!("{}", "┌───────────────────────────────────────────────────────────┐".magenta());
    println!(
        "{} {} {}",
    "│".magenta(),
        "⚙️   MANAGEMENT".bright_yellow().bold(),
        "                                           │".magenta()
    );
    println!("{}", "├───────────────────────────────────────────────────────────┤".magenta());
    println!(
        "{} {}",
        "│".magenta(),
        "  [5] ⚙️   Settings                                        │".bright_white()
    );
    println!(
        "{} {}",
        "│".magenta(),
        "  [q] ❌  Quit                                            │".bright_red()
    );
    println!("{}", "└───────────────────────────────────────────────────────────┘".magenta());
    println!("");
}

fn demander_url() -> String {
    loop {
        println!("{}", "═══════════════════════════════════════════════════════════".bright_blue());
        print!("{} ", "🔗  Entrez l'URL (YouTube, Soundcloud, etc.) :".bright_cyan().bold());
        io::stdout().flush().unwrap_or_else(|e| {
            error!("Erreur lors du flush stdout: {}", e);
        });

        let mut url = String::new();
        if io::stdin().read_line(&mut url).is_err() {
            error!("{}", "\n❌ Erreur de lecture de l'URL. Réessayez.\n".red().bold());
            continue;
        }
        let url = url.trim();
        if url.is_empty() {
            warn!("{}", "\n⚠️  L'URL ne peut pas être vide. Veuillez réessayer.\n".yellow().bold());
        } else {
            println!("{}", "═══════════════════════════════════════════════════════════\n".bright_blue());
            return url.to_string();
        }
    }
}