use crate::config;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::io;
use log::{info, warn};
use console::Term;

/// Fonction pour demander à l'utilisateur s'il souhaite personnaliser le nom du fichier à télécharger.
pub fn demander_nom_fichier_personnalise() -> Option<String> {
    info!("Souhaitez-vous personnaliser le nom du fichier à télécharger ? (o/n) :");
    let mut reponse = String::new();
    io::stdin().read_line(&mut reponse).expect("Erreur de lecture de la réponse de l'utilisateur");

    if reponse.trim().eq_ignore_ascii_case("o") {
        info!("Entrez le nom du fichier souhaité (sans l'extension) :");
        let mut nom_fichier = String::new();
        io::stdin().read_line(&mut nom_fichier).expect("Erreur de lecture du nom de fichier");
        let nom_fichier = nom_fichier.trim().to_string();

        if nom_fichier.is_empty() {
            warn!("Aucun nom personnalisé fourni, le nom par défaut sera utilisé.");
            None
        } else {
            Some(nom_fichier)
        }
    } else {
        None
    }
}

/// Fonction pour demander à l'utilisateur le format vidéo et s'il souhaite conserver les fichiers originaux après la fusion.
pub fn choisir_format_et_options() -> (String, bool) {
    let config = config::load_config();
    let default_format = config.default_video_format;
    let keep_files = config.keep_temporary_files;

    // Try to get a terminal instance for interactive mode
    let term = Term::stdout();
    
    // Check if we can use interactive mode
    if term.is_term() {
        // Use interactive mode with explicit terminal
        let format: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter the output format (leave empty for default)")
            .default(default_format)
            .interact_text()
            .unwrap_or_else(|_| "mp4".to_string());

        let keep_files = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Keep original files after merge?")
            .default(keep_files)
            .interact_opt()
            .unwrap_or(Some(false))
            .unwrap_or(false);

        (format, keep_files)
    } else {
        // Fallback to non-interactive mode
        warn!("Terminal interactif non disponible. Utilisation des options par défaut.");
        
        // Use default values
        let format = default_format;
        
        info!("Format vidéo sélectionné: {}", format);
        info!("Conserver les fichiers originaux: {}", if keep_files { "Oui" } else { "Non" });
        
        (format, keep_files)
    }
}

pub fn choisir_video_options_avances() -> (String, bool) {
    let resolutions = &["best", "1080p", "720p", "480p"];
    
    // Try to get a terminal instance for interactive mode
    let term = Term::stdout();
    
    // Check if we can use interactive mode
    if term.is_term() {
        // Use interactive mode with explicit terminal
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select video resolution")
            .items(resolutions)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        let format = match selection {
            Some(index) => resolutions[index].to_string(),
            None => "best".to_string(),
        };

        let keep_files = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Keep original files after merge?")
            .default(false)
            .interact_opt()
            .unwrap_or(Some(false))
            .unwrap_or(false);

        (format, keep_files)
    } else {
        // Fallback to non-interactive mode
        warn!("Terminal interactif non disponible. Utilisation des options par défaut.");
        
        // Print available resolutions for reference
        info!("Résolutions disponibles:");
        for (i, res) in resolutions.iter().enumerate() {
            info!("  {}. {}", i + 1, res);
        }
        
        // Use default values
        let format = "best".to_string();
        let keep_files = false;
        
        info!("Résolution sélectionnée: {}", format);
        info!("Conserver les fichiers originaux: {}", if keep_files { "Oui" } else { "Non" });
        
        (format, keep_files)
    }
}

/// Fonction pour demander à l'utilisateur le format audio.
pub fn choisir_audio_format() -> String {
    let config = config::load_config();
    let formats = &config.audio_formats;
    let default_format = &config.default_audio_format;

    let default_index = formats.iter().position(|f| f == default_format).unwrap_or(0);

    // Try to get a terminal instance for interactive mode
    let term = Term::stdout();
    
    // Check if we can use interactive mode
    if term.is_term() {
        // Use interactive mode with explicit terminal
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choisissez un format audio")
            .items(formats)
            .default(default_index)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(index) => formats[index].clone(),
            None => default_format.clone(),
        }
    } else {
        // Fallback to non-interactive mode
        warn!("Terminal interactif non disponible. Utilisation du format audio par défaut: {}", default_format);
        
        // Print available formats for reference
        info!("Formats audio disponibles:");
        for (i, format) in formats.iter().enumerate() {
            info!("  {}. {}", i + 1, format);
        }
        
        default_format.clone()
    }
}

/// Fonction pour demander à l'utilisateur s'il souhaite extraire uniquement la piste instrumentale.
/// Nécessite que Spleeter soit installé et accessible.
pub fn demander_extraction_instrumental(spleeter_available: bool) -> bool {
    if !spleeter_available {
        return false;
    }
    
    // Try to get a terminal instance for interactive mode
    let term = Term::stdout();
    
    // Check if we can use interactive mode
    if term.is_term() {
        // Use interactive mode with explicit terminal
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Voulez-vous extraire uniquement la piste instrumentale (sans les paroles) ?")
            .default(false)
            .interact_opt()
            .unwrap_or(Some(false))
            .unwrap_or(false)
    } else {
        // Fallback to non-interactive mode
        warn!("Terminal interactif non disponible. Extraction instrumentale désactivée par défaut.");
        false
    }
}

/// Fonction pour demander à l'utilisateur s'il souhaite continuer ou quitter le programme.
pub fn demander_si_continuer() -> bool {
    info!("Souhaitez-vous continuer à télécharger d'autres fichiers ? (o/n) :");
    let mut reponse = String::new();
    io::stdin().read_line(&mut reponse).expect("Erreur de lecture de la réponse de l'utilisateur");
    reponse.trim().eq_ignore_ascii_case("o")
}