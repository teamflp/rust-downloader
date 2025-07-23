use crate::config::{self, Config};
use colored::*;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use log::{info, warn, error};

pub fn show_settings_menu() {
    loop {
        let mut config = config::load_config();
        let menu_items = [
            "View Current Settings",
            "Set default video format",
            "Set download directory",
            "Toggle keep temporary files",
            "Manage audio formats",
            "Back to main menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Settings Menu")
            .items(&menu_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => view_current_settings(&config),
            Some(1) => set_default_video_format(&mut config),
            Some(2) => set_download_directory(&mut config),
            Some(3) => toggle_keep_temporary_files(&mut config),
            Some(4) => show_audio_formats_menu(),
            Some(5) => break,
            None => break,
            _ => (),
        }
    }
}


fn view_current_settings(config: &Config) {
    info!("{}", "Current Settings:".bold().underline());
    info!("- Default Video Format: {}", config.default_video_format.yellow());
    info!("- Download Directory: {}", config.download_directory.yellow());
    info!("- Keep Temporary Files: {}", if config.keep_temporary_files { "Yes".green() } else { "No".red() });
    info!("- Default Audio Format: {}", config.default_audio_format.yellow());
    info!("- Available Audio Formats: {}", config.audio_formats.join(", ").yellow());
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
        info!("{} {}", "Default video format set to:".green(), config.default_video_format.yellow());
    } else {
        warn!("{}", "Invalid input.".red());
    }
}

use std::path::Path;

fn set_download_directory(config: &mut Config) {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the new download directory")
        .default(config.download_directory.clone())
        .validate_with(|input: &String| -> Result<(), &str> {
            if Path::new(input).is_dir() {
                Ok(())
            } else {
                Err("This is not a valid directory.")
            }
        })
        .interact_text()
        .unwrap_or_else(|_| "".to_string());

    if !input.is_empty() {
        config.download_directory = input;
        config::save_config(config);
        info!("{} {}", "Download directory set to:".green(), config.download_directory.yellow());
    } else {
        warn!("{}", "Invalid input.".red());
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
        info!("{} {}", "Keep temporary files set to:".green(), if config.keep_temporary_files { "ON".green() } else { "OFF".red() });
    } else {
        warn!("{}", "No changes made.".yellow());
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
            Some(0) => info!("{} {}", "Current default audio format:".cyan(), config.default_audio_format.yellow()),
            Some(1) => set_default_format(&mut config),
            Some(2) => add_audio_format(&mut config),
            Some(3) => remove_audio_format(&mut config),
            Some(4) => break,
            None => break,
            _ => (),
        }
    }
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
        info!("{} {}", "Default audio format set to:".green(), config.default_audio_format.yellow());
    } else {
        warn!("{}", "No selection made.".yellow());
    }
}

fn add_audio_format(config: &mut Config) {
    let new_format: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the new audio format (e.g., opus)")
        .interact_text()
        .unwrap_or_else(|_| "".to_string());

    if new_format.is_empty() {
        warn!("{}", "Invalid format.".red());
        return;
    }

    if config.audio_formats.contains(&new_format) {
        warn!("{} {}", "Format already exists:".red(), new_format.yellow());
        return;
    }

    config.audio_formats.push(new_format.clone());
    config::save_config(config);
    info!("{} {}", "Added new format:".green(), new_format.yellow());
}

fn remove_audio_format(config: &mut Config) {
    if config.audio_formats.len() <= 1 {
        error!("{}", "You cannot remove the last audio format.".red());
        return;
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an audio format to remove")
        .items(&config.audio_formats)
        .default(0)
        .interact_opt()
        .unwrap_or(None);

    if let Some(index) = selection {
        let removed_format = config.audio_formats[index].clone();
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Are you sure you want to remove '{}'?", removed_format))
            .interact_opt()
            .unwrap_or(None);

        if let Some(true) = confirmation {
            config.audio_formats.remove(index);
            if removed_format == config.default_audio_format {
                config.default_audio_format = config.audio_formats[0].clone();
            }
            config::save_config(config);
            info!("{} {}", "Removed format:".green(), removed_format.yellow());
        } else {
            warn!("{}", "Removal cancelled.".yellow());
        }
    } else {
        warn!("{}", "No selection made.".yellow());
    }
}
