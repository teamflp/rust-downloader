use std::process::Stdio;
use std::process::{Command, exit};
use std::env;
use colored::*;

/// Vérifie si une commande système est disponible dans le PATH
fn is_command_available(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

use log::{info, warn, error};

/// Installe Homebrew (macOS) s'il est absent
fn install_brew() -> bool {
    info!("⚙️ Homebrew n'est pas installé. Installation en cours...");

    let cmd = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit()) // Show installation output
        .stderr(Stdio::inherit()) // Show installation error output
        .status()
        .expect("Erreur lors de l'exécution du script d'installation de Homebrew.");

    if !status.success() {
        error!("{}", "❌ L'installation de Homebrew a échoué.".red());
        error!("Veuillez installer Homebrew manuellement et réessayer.");
        return false;
    }

    info!("{}", "✅ Homebrew installé avec succès !".green());
    info!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal ou de sourcer votre fichier de profil (ex: ~/.zshrc, ~/.bash_profile) pour que brew soit pleinement utilisable.".yellow());
    true
}

/// Installe Chocolatey (Windows) s'il est absent
fn install_chocolatey() -> bool {
    info!("⚙️ Chocolatey n'est pas installé. Installation en cours...");

    let cmd = r#"Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#;

    let status = Command::new("powershell")
        .args(&["-NoProfile", "-InputFormat", "None", "-ExecutionPolicy", "Bypass", "-Command", cmd])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Erreur lors de l'exécution du script d'installation de Chocolatey.");

    if !status.success() {
        error!("{}", "❌ L'installation de Chocolatey a échoué.".red());
        error!("Veuillez installer Chocolatey manuellement et réessayer.");
        return false;
    }

    info!("{}", "✅ Chocolatey installé avec succès !".green());
    info!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal pour que choco soit pleinement utilisable.".yellow());
    true
}

/// Tentative d'installation via apt (Linux)
fn install_apt(package: &str) -> bool {
    let (command_name, base_args) = if env::var("USER").unwrap_or_default() == "root" {
        ("apt", Vec::new())
    } else if is_command_available("sudo") {
        ("sudo", vec!["apt"])
    } else {
        error!("{}", "❌ La commande 'sudo' est nécessaire pour 'apt' mais n'est pas trouvée et l'utilisateur n'est pas root.".red());
        error!("Veuillez installer le paquet '{}' manuellement ou exécuter en tant que root.", package);
        return false;
    };

    info!("⚙️ Mise à jour des dépôts apt (peut nécessiter un mot de passe)...");
    let mut update_args = base_args.clone();
    update_args.extend_from_slice(&["update", "-y"]);

    let update_status = Command::new(command_name)
        .args(&update_args)
        .status()
        .expect("Erreur lors de la mise à jour des dépôts apt.");

    if !update_status.success() {
        warn!("{}", "⚠️ Échec de la mise à jour des dépôts apt (apt update). Tentative d'installation quand même...".yellow());
    }

    info!("⚙️ Tentative d'installation de '{}' avec apt (peut nécessiter un mot de passe)...", package);
    let mut install_args = base_args;
    install_args.extend_from_slice(&["install", "-y", package]);

    let status = Command::new(command_name)
        .args(&install_args)
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec apt.", package));

    if !status.success() {
        error!("❌ L'installation de {} avec apt a échoué.", package.red());
        return false;
    }
    info!("✅ {} installé avec succès via apt!", package.green());
    true
}

/// Tentative d'installation via brew (macOS)
fn install_brew_package(package: &str) -> bool {
    if !is_command_available("brew") {
        if !install_brew() {
            return false;
        }
    }
    info!("⚙️ Tentative d'installation de '{}' avec brew...", package);
    let status = Command::new("brew")
        .args(&["install", package])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'exécution de brew install {}.", package));
    if !status.success() {
        error!("❌ L'installation de {} avec brew a échoué.", package.red());
        return false;
    }
    info!("✅ {} installé avec succès via brew!", package.green());
    true
}

/// Tentative d'installation via choco (Windows)
fn install_choco_package(package: &str) -> bool {
    if !is_command_available("choco") {
        if !install_chocolatey() {
            return false;
        }
    }
    info!("⚙️ Tentative d'installation de '{}' avec choco...", package);
    let status = Command::new("choco")
        .args(&["install", package, "-y"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'exécution de choco install {}.", package));
    if !status.success() {
        error!("❌ L'installation de {} avec choco a échoué.", package.red());
        return false;
    }
    info!("✅ {} installé avec succès via choco!", package.green());
    true
}

/// Tentative d'installation via scoop (Windows)
fn install_scoop_package(package: &str) -> bool {
    if !is_command_available("scoop") {
        info!("⚙️ Scoop n'est pas installé. Tentative d'installation...");
        let cmd = "Set-ExecutionPolicy RemoteSigned -Scope CurrentUser; irm get.scoop.sh | iex";
        let status = Command::new("powershell")
            .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Erreur lors de l'installation de Scoop.");

        if !status.success() {
            error!("{}", "❌ L'installation de Scoop a échoué.".red());
            return false;
        }
        info!("{}", "✅ Scoop installé avec succès!".green());
        info!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal pour que scoop soit pleinement utilisable.".yellow());
    }
    info!("⚙️ Tentative d'installation de '{}' avec scoop...", package);
    let status = Command::new("scoop")
        .args(&["install", package])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec scoop.", package));
    if !status.success() {
        error!("❌ L'installation de {} avec scoop a échoué.", package.red());
        return false;
    }
    info!("✅ {} installé avec succès via scoop!", package.green());
    true
}

/// Installe ffmpeg de manière multiplateforme
pub fn install_ffmpeg() {
    info!("⚙️ Installation de ffmpeg...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => install_apt("ffmpeg"),
        "macos" => install_brew_package("ffmpeg"),
        "windows" => install_choco_package("ffmpeg") || install_scoop_package("ffmpeg"),
        _ => {
            error!("❌ Système d'exploitation '{}' non supporté pour l'installation automatique de ffmpeg.", os.red());
            false
        }
    };

    if !success {
        error!("{}", "❌ L'installation de ffmpeg a échoué. Veuillez l'installer manuellement.".red());
        exit(1);
    }
    info!("{}", "✅ ffmpeg est maintenant prêt.".green());
}

/// Helper function for installing yt-dlp on Linux
fn install_yt_dlp_linux_internal() -> bool {
    let (sudo_prefix, _is_root) = if env::var("USER").unwrap_or_default() == "root" {
        ("", true)
    } else if is_command_available("sudo") {
        ("sudo ", false)
    } else {
        error!("{}", "❌ 'sudo' est requis pour installer yt-dlp globalement mais n'est pas trouvé.".red());
        return false;
    };

    let install_dir = "/usr/local/bin";
    let yt_dlp_path = format!("{}/yt-dlp", install_dir);

    info!("Téléchargement de yt-dlp vers {}...", yt_dlp_path);
    let cmd_dl = format!(
        "{}curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o {}",
        sudo_prefix, yt_dlp_path
    );
    let status_dl = Command::new("sh").arg("-c").arg(&cmd_dl).status();

    if !status_dl.map_or(false, |s| s.success()) {
        error!("{}", "❌ Échec du téléchargement de yt-dlp.".red());
        return false;
    }

    info!("Configuration des permissions pour {}...", yt_dlp_path);
    let cmd_chmod = format!("{}chmod a+rx {}", sudo_prefix, yt_dlp_path);
    let status_chmod = Command::new("sh").arg("-c").arg(&cmd_chmod).status();

    if !status_chmod.map_or(false, |s| s.success()) {
        error!("{}", "❌ Échec de la configuration des permissions pour yt-dlp.".red());
        if sudo_prefix == "sudo " { // Attempt to clean up only if sudo was used for download
            let _ = Command::new("sh").arg("-c").arg(format!("{}rm -f {}", sudo_prefix, yt_dlp_path)).status();
        }
        return false;
    }
    info!("{}", "✅ yt-dlp installé avec succès via curl!".green());
    true
}


/// Installe yt-dlp de manière multiplateforme
pub fn install_yt_dlp() {
    info!("⚙️ Installation de yt-dlp...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => install_yt_dlp_linux_internal(),
        "macos" => install_brew_package("yt-dlp"),
        "windows" => install_choco_package("yt-dlp") || install_scoop_package("yt-dlp"),
        _ => {
            error!("❌ Système d'exploitation '{}' non supporté pour l'installation automatique de yt-dlp.", os.red());
            false
        }
    };

    if !success {
        error!("{}", "❌ L'installation de yt-dlp a échoué. Veuillez l'installer manuellement.".red());
        exit(1);
    }
    info!("{}", "✅ yt-dlp est maintenant prêt.".green());
}

// This file no longer contains Spleeter installation logic.

/// Vérifie et installe tous les outils nécessaires
pub fn ensure_dependencies() {
    info!("{}", "🔍 Vérification des dépendances...".bold());

    if !is_command_available("ffmpeg") {
        install_ffmpeg(); // This function calls exit(1) on failure
    } else {
        info!("{}", "✅ ffmpeg est déjà installé.".green());
    }

    if !is_command_available("yt-dlp") {
        install_yt_dlp(); // This function calls exit(1) on failure
    } else {
        info!("{}", "✅ yt-dlp est déjà installé.".green());
    }
}