use std::process::{Command, exit};

pub fn install_ffmpeg() {
    println!("ffmpeg n'est pas installé. Installation en cours...");

    if cfg!(target_os = "macos") {
        let status = Command::new("sh")
            .arg("-c")
            .arg("brew install ffmpeg")
            .status()
            .expect("Erreur lors de l'installation de ffmpeg avec Homebrew.");
        if !status.success() {
            eprintln!("L'installation de ffmpeg a échoué.");
            exit(1);
        }
    } else if cfg!(target_os = "linux") {
        let status = Command::new("sh")
            .arg("-c")
            .arg("sudo apt update && sudo apt install -y ffmpeg")
            .status()
            .expect("Erreur lors de l'installation de ffmpeg sur Linux.");
        if !status.success() {
            eprintln!("L'installation de ffmpeg a échoué.");
            exit(1);
        }
    } else if cfg!(target_os = "windows") {
        let status = Command::new("powershell")
            .args(&["-Command", "choco install ffmpeg -y"])
            .status()
            .expect("Erreur lors de l'installation de ffmpeg avec Chocolatey.");
        if !status.success() {
            eprintln!("L'installation de ffmpeg a échoué.");
            exit(1);
        }
    } else {
        eprintln!("Système d'exploitation non supporté pour l'installation automatique de ffmpeg.");
        exit(1);
    }
}

pub fn install_yt_dlp() {
    println!("yt-dlp n'est pas installé. Installation en cours...");

    if cfg!(target_os = "macos") {
        let status = Command::new("sh")
            .arg("-c")
            .arg("brew install yt-dlp")
            .status()
            .expect("Erreur lors de l'installation de yt-dlp avec Homebrew.");
        if !status.success() {
            eprintln!("L'installation de yt-dlp a échoué.");
            exit(1);
        }
    } else if cfg!(target_os = "linux") {
        let status = Command::new("sh")
            .arg("-c")
            .arg("sudo curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp")
            .status()
            .expect("Erreur lors de l'installation de yt-dlp sur Linux.");
        if !status.success() {
            eprintln!("L'installation de yt-dlp a échoué.");
            exit(1);
        }
    } else if cfg!(target_os = "windows") {
        let status = Command::new("powershell")
            .args(&["-Command", "choco install yt-dlp -y"])
            .status()
            .expect("Erreur lors de l'installation de yt-dlp avec Chocolatey.");
        if !status.success() {
            eprintln!("L'installation de yt-dlp a échoué.");
            exit(1);
        }
    } else {
        eprintln!("Système d'exploitation non supporté pour l'installation automatique de yt-dlp.");
        exit(1);
    }
}
