use crate::config::{self, Config};
use colored::*;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};

pub fn show_settings_menu() {
    loop {
        let mut config = config::load_config();
         let menu_items = [
            "Afficher les paramètres actuels",
            "Définir le format vidéo par défaut",
            "Définir le répertoire de téléchargement",
            "Activer/Désactiver la conservation des fichiers temporaires",
            "Gérer les formats audio", // Cette option mène à la logique de la branche `master`.
            "Retour au menu principal",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Menu des Paramètres ⚙️")
            .items(&menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            // Affiche tous les paramètres actuels.
            Some(0) => view_current_settings(&config),
            // Définit le format vidéo par défaut.
            Some(1) => set_default_video_format(&mut config),
            // Définit le dossier de téléchargement.
            Some(2) => set_download_directory(&mut config),
            // Bascule l'option de conservation des fichiers.
            Some(3) => toggle_keep_temporary_files(&mut config),
            // Ouvre le sous-menu pour la gestion des formats audio.
            Some(4) => show_audio_formats_menu(),
            // L'index 5 correspond maintenant à "Retour au menu principal".
            Some(5) => break,
            // L'utilisateur a appuyé sur Echap ou Q.
            None => break,
            // Cas par défaut qui ne devrait pas arriver.
            _ => (),
        }
    }
}

fn view_current_settings(config: &Config) {
    println!("{}", "Current Settings:".bold().underline());
    println!("- Default Video Format: {}", config.default_video_format.yellow());
    println!("- Download Directory: {}", config.download_directory.yellow());
    println!("- Keep Temporary Files: {}", if config.keep_temporary_files { "Yes".green() } else { "No".red() });
    println!("- Default Audio Format: {}", config.default_audio_format.yellow());
    println!("- Available Audio Formats: {}", config.audio_formats.join(", ").yellow());
}

fn set_default_video_format(config: &mut Config) {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the new default video format (e.g., mp4, webm)")
        .default(config.default_video_format.clone())
        .interact_text()
        .unwrap_or_else(|_| "".to_string());

    if !input.is_empty() {
        config.default_video_format = input;
        config::save_config(config);
        println!("{} {}", "Default video format set to:".green(), config.default_video_format.yellow());
    } else {
        println!("{}", "Invalid input.".red());
    }
}

fn set_download_directory(config: &mut Config) {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the new download directory")
        .default(config.download_directory.clone())
        .interact_text()
        .unwrap_or_else(|_| "".to_string());

    if !input.is_empty() {
        config.download_directory = input;
        config::save_config(config);
        println!("{} {}", "Download directory set to:".green(), config.download_directory.yellow());
    } else {
        println!("{}", "Invalid input.".red());
    }
}

fn toggle_keep_temporary_files(config: &mut Config) {
    let current_status = config.keep_temporary_files;
    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Keep temporary files is currently {}. Do you want to toggle it?", if current_status { "ON".green() } else { "OFF".red() }))
        .interact_opt()
        .unwrap_or(None);

    if let Some(true) = confirmation {
        config.keep_temporary_files = !current_status;
        config::save_config(config);
        println!("{} {}", "Keep temporary files set to:".green(), if config.keep_temporary_files { "ON".green() } else { "OFF".red() });
    } else {
        println!("{}", "No changes made.".yellow());
    }
}

fn show_audio_formats_menu() {
    loop {
        let mut config = config::load_config();
        let menu_items = [
            "View default audio format",
            "Set default audio format",
            "Add an audio format",
            "Remove an audio format",
            "Back to settings menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Audio Formats Menu")
            .items(&menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => println!("{} {}", "Current default audio format:".cyan(), config.default_audio_format.yellow()),
            Some(1) => set_default_format(&mut config),
            Some(2) => add_audio_format(&mut config),
            Some(3) => remove_audio_format(&mut config),
            Some(4) => break,
            None => break,
            _ => (),
        }
    }

fn view_default_format(config: &Config) {
    println!("{} {}", "Current default audio format:".cyan(), config.default_audio_format.yellow());

}

fn set_default_format(config: &mut Config) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a new default audio format")
        .items(&config.audio_formats)
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    if let Some(index) = selection {
        config.default_audio_format = config.audio_formats[index].clone();
        config::save_config(config);
        println!("{} {}", "Default audio format set to:".green(), config.default_audio_format.yellow());
    } else {
        println!("{}", "No selection made.".yellow());
    }
}

fn add_audio_format(config: &mut Config) {
    let new_format: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the new audio format (e.g., opus)")
        .interact_text()
        .unwrap_or_else(|_| "".to_string());

    if new_format.is_empty() {
        println!("{}", "Invalid format.".red());
        return;
    }

    if config.audio_formats.contains(&new_format) {
        println!("{} {}", "Format already exists:".red(), new_format.yellow());
        return;
    }

    config.audio_formats.push(new_format.clone());
    config::save_config(config);
    println!("{} {}", "Added new format:".green(), new_format.yellow());
}

fn remove_audio_format(config: &mut Config) {
    if config.audio_formats.len() <= 1 {
        println!("{}", "You cannot remove the last audio format.".red());
        return;
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an audio format to remove")
        .items(&config.audio_formats)
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    if let Some(index) = selection {
        let removed_format = config.audio_formats.remove(index);
        if removed_format == config.default_audio_format {
            config.default_audio_format = config.audio_formats[0].clone();
        }
        config::save_config(config);
        println!("{} {}", "Removed format:".green(), removed_format.yellow());
    } else {
        println!("{}", "No selection made.".yellow());
    }
}
