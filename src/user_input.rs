use std::io;

/// Fonction pour demander à l'utilisateur s'il souhaite personnaliser le nom du fichier à télécharger.
pub fn demander_nom_fichier_personnalise() -> Option<String> {
    println!("Souhaitez-vous personnaliser le nom du fichier à télécharger ? (o/n) :");
    let mut reponse = String::new();
    io::stdin().read_line(&mut reponse).expect("Erreur de lecture de la réponse de l'utilisateur");

    if reponse.trim().eq_ignore_ascii_case("o") {
        println!("Entrez le nom du fichier souhaité (sans l'extension) :");
        let mut nom_fichier = String::new();
        io::stdin().read_line(&mut nom_fichier).expect("Erreur de lecture du nom de fichier");
        let nom_fichier = nom_fichier.trim().to_string();

        if nom_fichier.is_empty() {
            println!("Aucun nom personnalisé fourni, le nom par défaut sera utilisé.");
            None
        } else {
            Some(nom_fichier)
        }
    } else {
        None
    }
}

use dialoguer::{Confirm, Select, theme::ColorfulTheme};

/// Fonction pour demander à l'utilisateur le format vidéo et s'il souhaite conserver les fichiers originaux après la fusion.
pub fn choisir_format_et_options() -> (String, bool) {
    let config = config::load_config();
    let default_format = config.default_video_format;
    let keep_files = config.keep_temporary_files;

    let format: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
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
}

use crate::config;

/// Fonction pour demander à l'utilisateur le format audio.
pub fn choisir_audio_format() -> String {
    let config = config::load_config();
    let formats = &config.audio_formats;
    let default_format = &config.default_audio_format;

    let default_index = formats.iter().position(|f| f == default_format).unwrap_or(0);

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
}

/// Fonction pour demander à l'utilisateur s'il souhaite extraire uniquement la piste instrumentale.
/// Nécessite que Spleeter soit installé et accessible.
pub fn demander_extraction_instrumental() -> bool {
    println!("Voulez-vous extraire uniquement la piste instrumentale (sans les paroles) ? (o/n)");
    println!("Note : Ceci nécessite que Spleeter soit installé sur votre système.");
    let mut reponse = String::new();
    io::stdin().read_line(&mut reponse).expect("Erreur de lecture de la réponse de l'utilisateur");
    reponse.trim().eq_ignore_ascii_case("o")
}

/// Fonction pour demander à l'utilisateur s'il souhaite continuer ou quitter le programme.
pub fn demander_si_continuer() -> bool {
    println!("Souhaitez-vous continuer à télécharger d'autres fichiers ? (o/n) :");
    let mut reponse = String::new();
    io::stdin().read_line(&mut reponse).expect("Erreur de lecture de la réponse de l'utilisateur");
    reponse.trim().eq_ignore_ascii_case("o")
}
