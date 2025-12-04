use which::which;
use std::process::Command;
use log::{info, error};

pub fn check_command(cmd: &str) -> bool {
    which(cmd).is_ok()
}

pub fn install_spleeter() -> bool {
    info!("Tentative d'installation automatique de Spleeter via pip...");

    if !check_command("pip") {
        error!("❌ 'pip' n'est pas installé. Impossible d'installer Spleeter automatiquement.");
        return false;
    }

    let status = Command::new("pip")
        .args(&["install", "spleeter"])
        .status();

    match status {
        Ok(s) if s.success() => {
            info!("✅ Spleeter a été installé avec succès !");
            true
        }
        Ok(s) => {
            error!("❌ Échec de l'installation de Spleeter. Code de sortie : {:?}", s.code());
            false
        }
        Err(e) => {
            error!("❌ Erreur lors de l'exécution de pip : {}", e);
            false
        }
    }
}
