use colored::*;
use dialoguer::{Select, theme::ColorfulTheme};
use std::process::{Command, Stdio};
use which::which;
use log::{info, warn, error};

// Liste des navigateurs supportés et leurs noms pour yt-dlp
const BROWSERS: &[(&str, &str)] = &[
    ("chrome", "Chrome"),
    ("firefox", "Firefox"),
    ("brave", "Brave"),
    ("edge", "Edge"),
    ("opera", "Opera"),
    ("vivaldi", "Vivaldi"),
    // Safari n'est pas directement supporté pour l'extraction de cookies par yt-dlp de cette manière.
];

/// Détecte les navigateurs installés sur le système.
/// Retourne une liste de tuples `(key, name)` pour les navigateurs trouvés.
pub fn get_installed_browsers() -> Vec<(&'static str, &'static str)> {
    let mut installed = Vec::new();
    for (key, name) in BROWSERS {
        // Pour Windows, les navigateurs ne sont pas toujours dans le PATH.
        // yt-dlp a sa propre logique de détection interne, donc on les ajoute tous sur Windows.
        if cfg!(target_os = "windows") {
            installed.push((*key, *name));
        } else {
            // Pour Linux/macOS, on peut vérifier la présence de l'exécutable.
            if which(key).is_ok() {
                installed.push((*key, *name));
            }
        }
    }
    installed
}

/// Affiche un menu pour choisir un navigateur et exécute le téléchargement.
pub fn extract_cookies_and_download(url: &str) {
    if which("yt-dlp").is_err() {
        error!("{}", "Erreur: 'yt-dlp' n'est pas installé ou pas dans le PATH.".red().bold());
        return;
    }

    let browsers = get_installed_browsers();
    if browsers.is_empty() {
        error!("{}", "Aucun navigateur compatible n'a été détecté.".red().bold());
        info!("Navigateurs supportés : Chrome, Firefox, Brave, Edge, Opera, Vivaldi.");
        return;
    }

    let browser_names: Vec<&str> = browsers.iter().map(|&(_, name)| name).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choisissez un navigateur pour extraire les cookies")
        .items(&browser_names)
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    if let Some(index) = selection {
        let (browser_key, browser_name) = browsers[index];
        info!("{} Utilisation des cookies de {}...", "🔑".yellow(), browser_name.cyan());

        download_with_cookies(url, browser_key);
    } else {
        warn!("{}", "Aucun navigateur sélectionné. Annulation.".yellow());
    }
}

/// Exécute yt-dlp avec les cookies du navigateur spécifié.
fn download_with_cookies(url: &str, browser: &str) {
    info!("{}", "\n📥 Téléchargement en cours...".cyan().bold());

    let mut command = Command::new("yt-dlp");
    command
        .arg("--cookies-from-browser")
        .arg(browser)
        .arg(url)
        .stdout(Stdio::inherit()) // Affiche la sortie de yt-dlp en temps réel
        .stderr(Stdio::inherit()); // Affiche les erreurs de yt-dlp en temps réel

    match command.status() {
        Ok(status) => {
            if status.success() {
                info!("{}", "\n✅ Téléchargement terminé avec succès !".green().bold());
            } else {
                error!(
                    "{}",
                    "\n❌ Erreur lors du téléchargement. yt-dlp a retourné un code d'erreur."
                        .red()
                        .bold()
                );
            }
        }
        Err(e) => {
            error!(
                "{}\nErreur : {}",
                "❌ Échec de l'exécution de la commande yt-dlp.".red().bold(),
                e
            );
        }
    }
}