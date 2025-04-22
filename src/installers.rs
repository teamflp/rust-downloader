use std::process::{Command, exit};
use std::env;

/// Vérifie si une commande système est disponible dans le PATH
fn is_command_available(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Installe Homebrew (macOS) s'il est absent
fn install_brew() -> bool {
    println!("⚙️ Homebrew n'est pas installé. Installation en cours...");

    let cmd = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .expect("Erreur lors de l'installation de Homebrew.");

    if !status.success() {
        eprintln!("❌ L'installation de Homebrew a échoué.");
        return false;
    }

    println!("✅ Homebrew installé avec succès !");
    true
}

/// Installe Chocolatey (Windows) s'il est absent
fn install_chocolatey() -> bool {
    println!("⚙️ Chocolatey n'est pas installé. Installation en cours...");

    let cmd = r#"Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#;

    let status = Command::new("powershell")
        .args(&["-NoProfile", "-InputFormat", "None", "-ExecutionPolicy", "Bypass", "-Command", cmd])
        .status()
        .expect("Erreur lors de l'installation de Chocolatey.");

    if !status.success() {
        eprintln!("❌ L'installation de Chocolatey a échoué.");
        return false;
    }

    println!("✅ Chocolatey installé avec succès !");
    true
}

/// Tentative d'installation via apt (Linux)
fn install_apt(package: &str) -> bool {
    let sudo = if is_command_available("sudo") { "sudo" } else { "" };
    let update_status = Command::new(sudo)
        .args(&["apt", "update"])
        .status()
        .expect("Erreur lors de la mise à jour des dépôts apt.");

    if !update_status.success() {
        eprintln!("❌ Échec de la mise à jour des dépôts apt.");
        return false;
    }
    let status = Command::new(sudo)
        .args(&["apt", "install", "-y", package])
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec apt.", package));

    if !status.success() {
        eprintln!("❌ L'installation de {} a échoué.", package);
        return false;
    }
    println!("✅ {} installé avec succès via apt!", package);
    true
}

/// Tentative d'installation via brew (macOS)
fn install_brew_package(package: &str) -> bool {
    if !is_command_available("brew") {
        if !install_brew() {
            return false;
        }
    }
    let status = Command::new("brew")
        .args(&["install", package])
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec brew.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} a échoué.", package);
        return false;
    }
    println!("✅ {} installé avec succès via brew!", package);
    true
}

/// Tentative d'installation via choco (Windows)
fn install_choco_package(package: &str) -> bool {
    if !is_command_available("choco") {
        if !install_chocolatey() {
            return false;
        }
    }
    let status = Command::new("choco")
        .args(&["install", package, "-y"])
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec choco.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} a échoué.", package);
        return false;
    }
    println!("✅ {} installé avec succès via choco!", package);
    true
}

/// Tentative d'installation via scoop (Windows)
fn install_scoop_package(package: &str) -> bool {
    if !is_command_available("scoop") {
        println!("⚙️ Scoop n'est pas installé. Installation en cours...");
        let cmd = r#"powershell -NoProfile -ExecutionPolicy Bypass -Command "(New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh') | iex""#;
        let status = Command::new("powershell")
            .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
            .status()
            .expect("Erreur lors de l'installation de Scoop.");

        if !status.success() {
            eprintln!("❌ L'installation de Scoop a échoué.");
            return false;
        }
        println!("✅ Scoop installé avec succès!");
    }
    let status = Command::new("scoop")
        .args(&["install", package])
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec scoop.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} a échoué.", package);
        return false;
    }
    println!("✅ {} installé avec succès via scoop!", package);
    true
}

/// Installe ffmpeg de manière multiplateforme
pub fn install_ffmpeg() {
    println!("⚙️ Installation de ffmpeg...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => install_apt("ffmpeg"),
        "macos" => install_brew_package("ffmpeg"),
        "windows" => install_choco_package("ffmpeg") || install_scoop_package("ffmpeg"),
        _ => {
            eprintln!("❌ Système non supporté pour installer ffmpeg.");
            false
        }
    };

    if !success {
        eprintln!("❌ L'installation de ffmpeg a échoué sur ce système.");
        exit(1);
    }
}

/// Installe yt-dlp de manière multiplateforme
pub fn install_yt_dlp() {
    println!("⚙️ Installation de yt-dlp...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => {
            let sudo = if is_command_available("sudo") { "sudo " } else { "" };
            let cmd = format!(
                "{sudo}curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && {sudo}chmod a+rx /usr/local/bin/yt-dlp"
            );
            let status = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .status()
                .expect("Erreur lors de l'installation de yt-dlp.");
            status.success()
        },
        "macos" => install_brew_package("yt-dlp"),
        "windows" => install_choco_package("yt-dlp") || install_scoop_package("yt-dlp"),
        _ => {
            eprintln!("❌ Système non supporté pour installer yt-dlp.");
            false
        }
    };

    if !success {
        eprintln!("❌ L'installation de yt-dlp a échoué sur ce système.");
        exit(1);
    }
}

/// Vérifie et installe tous les outils nécessaires
pub fn ensure_dependencies() {
    println!("🔍 Vérification des dépendances...");

    if !is_command_available("ffmpeg") {
        install_ffmpeg();
    } else {
        println!("✅ ffmpeg est déjà installé.");
    }

    if !is_command_available("yt-dlp") {
        install_yt_dlp();
    } else {
        println!("✅ yt-dlp est déjà installé.");
    }

    println!("🎉 Toutes les dépendances sont prêtes !");
}
