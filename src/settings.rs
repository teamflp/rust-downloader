use crate::config::{self, Config};
use colored::*;
use dialoguer::{Input, Select, theme::ColorfulTheme};

pub fn show_settings_menu() {
    loop {
        let config = config::load_config();
        let menu_items = [
            "View current default audio format",
            "Set a new default audio format",
            "Add a new audio format to the list",
            "Remove an audio format from the list",
            "Back to main menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Settings Menu")
            .items(&menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => view_default_format(&config),
            Some(1) => set_default_format(&mut config.clone()),
            Some(2) => add_audio_format(&mut config.clone()),
            Some(3) => remove_audio_format(&mut config.clone()),
            Some(4) => break,
            None => break,
            _ => (),
        }
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
