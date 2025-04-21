use std::process::{Command, exit};

/// Vérifie si une commande système est disponible dans le PATH
fn is_command_available(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Installe Homebrew (macOS) s'il est absent
fn install_brew() {
    println!("⚙️ Homebrew n'est pas installé. Installation en cours...");

    let cmd = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .expect("Erreur lors de l'installation de Homebrew.");

    if !status.success() {
        eprintln!("❌ L'installation de Homebrew a échoué.");
        exit(1);
    }

    println!("✅ Homebrew installé avec succès !");
}

/// Installe Chocolatey (Windows) s'il est absent
fn install_chocolatey() {
    println!("⚙️ Chocolatey n'est pas installé. Installation en cours...");

    let cmd = r#"Set-ExecutionPolicy Bypass -Scope Process -Force; `
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; `
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#;

    let status = Command::new("powershell")
        .args(&["-NoProfile", "-InputFormat", "None", "-ExecutionPolicy", "Bypass", "-Command", cmd])
        .status()
        .expect("Erreur lors de l'installation de Chocolatey.");

    if !status.success() {
        eprintln!("❌ L'installation de Chocolatey a échoué.");
        exit(1);
    }

    println!("✅ Chocolatey installé avec succès !");
}

/// Installe ffmpeg
pub fn install_ffmpeg() {
    println!("⚙️ Installation de ffmpeg...");

    let mut command = if cfg!(target_os = "macos") {
        if !is_command_available("brew") {
            install_brew();
        }
        Command::new("brew")
    } else if cfg!(target_os = "linux") {
        let sudo = if is_command_available("sudo") { "sudo " } else { "" };
        let _install_cmd = format!("{sudo}apt update && {sudo}apt install -y ffmpeg");
        Command::new("sh")
    } else if cfg!(target_os = "windows") {
        if !is_command_available("choco") {
            install_chocolatey();
        }
        Command::new("choco")
    } else {
        eprintln!("❌ Système non supporté pour installer ffmpeg.");
        exit(1);
    };

    let status = if cfg!(target_os = "macos") {
        command.arg("install").arg("ffmpeg").status()
    } else if cfg!(target_os = "linux") {
        command.arg("-c").arg(format!("{sudo}apt update && {sudo}apt install -y ffmpeg", sudo = if is_command_available("sudo") { "sudo " } else { "" })).status()
    } else if cfg!(target_os = "windows") {
        command.arg("install").arg("ffmpeg").arg("-y").status()
    } else {
        unreachable!(); // Déjà géré par le bloc else précédent avec exit(1)
    }
        .expect("Erreur lors de l'installation de ffmpeg.");

    if status.success() {
        println!("✅ ffmpeg installé avec succès !");
    } else {
        eprintln!("❌ L'installation de ffmpeg a échoué.");
        exit(1);
    }
}

/// Installe yt-dlp
pub fn install_yt_dlp() {
    println!("⚙️ Installation de yt-dlp...");

    let mut command = if cfg!(target_os = "macos") {
        if !is_command_available("brew") {
            install_brew();
        }
        Command::new("brew")
    } else if cfg!(target_os = "linux") {
        let sudo = if is_command_available("sudo") { "sudo " } else { "" };
        let _cmd = format!(
            "{sudo}curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && {sudo}chmod a+rx /usr/local/bin/yt-dlp"
        );
        Command::new("sh")
    } else if cfg!(target_os = "windows") {
        if !is_command_available("choco") {
            install_chocolatey();
        }
        Command::new("choco")
    } else {
        eprintln!("❌ Système non supporté pour installer yt-dlp.");
        exit(1);
    };

    let status = if cfg!(target_os = "macos") {
        command.arg("install").arg("yt-dlp").status()
    } else if cfg!(target_os = "linux") {
        command.arg("-c").arg(format!(
            "{sudo}curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && {sudo}chmod a+rx /usr/local/bin/yt-dlp",
            sudo = if is_command_available("sudo") { "sudo " } else { "" }
        )).status()
    } else if cfg!(target_os = "windows") {
        command.arg("install").arg("yt-dlp").arg("-y").status()
    } else {
        unreachable!(); // Déjà géré par le bloc else précédent avec exit(1)
    }
        .expect("Erreur lors de l'installation de yt-dlp.");

    if status.success() {
        println!("✅ yt-dlp installé avec succès !");
    } else {
        eprintln!("❌ L'installation de yt-dlp a échoué.");
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