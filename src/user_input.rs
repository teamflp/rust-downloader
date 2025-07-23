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

/// Fonction pour demander à l'utilisateur le format vidéo et s'il souhaite conserver les fichiers originaux après la fusion.
pub fn choisir_format_et_options() -> (String, bool) {
    println!("Entrez le format de sortie (ex. 'mp4', 'webm', laissez vide pour le format par défaut) :");
    let mut format = String::new();
    io::stdin().read_line(&mut format).expect("Erreur de lecture du format de sortie");
    let format = format.trim().to_string();

    println!("Voulez-vous conserver les fichiers originaux après la fusion ? (o/n) :");
    let mut keep_files_input = String::new();
    io::stdin().read_line(&mut keep_files_input).expect("Erreur de lecture du choix de l'utilisateur");
    let keep_files = keep_files_input.trim().eq_ignore_ascii_case("o");

    (format, keep_files)
}

use crate::config;
use dialoguer::{Select, theme::ColorfulTheme};

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
