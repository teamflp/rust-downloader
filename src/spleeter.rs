use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{BufReader, AsyncBufReadExt};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn, error};
use anyhow::{Result, Context};

pub async fn extract_instrumental(original_downloaded_full_path: &PathBuf) -> Result<()> {
    info!("⚙️  Extraction de l'instrumental avec Spleeter en cours (cela peut prendre du temps)...");

    if Command::new("spleeter").arg("--version").output().await.is_err() {
        error!("❌ Spleeter n'est pas installé ou n'est pas dans le PATH.");
        info!("   Veuillez l'installer pour utiliser l'extraction instrumentale.");
        info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
        return Ok(());
    }

    let input_audio_path_for_spleeter = original_downloaded_full_path
        .to_str()
        .context("Chemin du fichier audio original invalide pour Spleeter")?;

    let spleeter_output_parent_dir = original_downloaded_full_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    info!("Spleeter utilisera le dossier de sortie : {:?}", spleeter_output_parent_dir);

    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap();
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.set_message("Spleeter is working...");

    let mut spleeter_cmd = Command::new("spleeter")
        .arg("separate")
        .arg("-p")
        .arg("spleeter:2stems") // Separates into vocals and accompaniment
        .arg("-o")
        .arg(spleeter_output_parent_dir) // Spleeter creates a subfolder here
        .arg(input_audio_path_for_spleeter)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Erreur lors du lancement de Spleeter")?;

    info!("Spleeter démarré...");
    
    // Capture Spleeter's output
    if let Some(s_stdout) = spleeter_cmd.stdout.take() {
        let reader = BufReader::new(s_stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            info!("Spleeter (stdout): {}", line);
        }
    }
    if let Some(s_stderr) = spleeter_cmd.stderr.take() {
        let reader = BufReader::new(s_stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            error!("Spleeter (stderr): {}", line);
        }
    }

    let spleeter_status = spleeter_cmd.wait().await.context("Spleeter a échoué lors de l'attente")?;
    pb.finish_with_message("Spleeter finished.");

    if spleeter_status.success() {
        info!("✅ Spleeter a terminé l'extraction.");

        let original_file_stem = original_downloaded_full_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("audio_file");

        // Spleeter creates a directory named after the input file's stem
        let spleeter_output_subdir = spleeter_output_parent_dir.join(original_file_stem);
        let instrumental_spleeter_filename = "accompaniment.wav"; // Default for 2stems
        let vocals_spleeter_filename = "vocals.wav";

        let spleeter_instrumental_path = spleeter_output_subdir.join(instrumental_spleeter_filename);
        let spleeter_vocals_path = spleeter_output_subdir.join(vocals_spleeter_filename);

        if spleeter_instrumental_path.exists() {
            let final_instrumental_filename = format!("{}_instrumental.wav", original_file_stem);
            let final_instrumental_full_path = original_downloaded_full_path.with_file_name(final_instrumental_filename);

            match fs::rename(&spleeter_instrumental_path, &final_instrumental_full_path) {
                Ok(_) => {
                    info!("🎶 Fichier instrumental sauvegardé ici : {:?}", final_instrumental_full_path);
                    // Cleanup
                    if let Err(e) = fs::remove_file(original_downloaded_full_path) {
                        warn!("⚠️ Impossible de supprimer le fichier audio original complet {:?}: {}", original_downloaded_full_path, e);
                    }
                    if spleeter_vocals_path.exists() {
                        if let Err(e) = fs::remove_file(&spleeter_vocals_path) {
                            warn!("⚠️ Impossible de supprimer le fichier vocal {:?}: {}", spleeter_vocals_path, e);
                        }
                    }
                    if spleeter_output_subdir.exists() {
                        if let Err(e) = fs::remove_dir_all(&spleeter_output_subdir) {
                            warn!("⚠️ Impossible de supprimer le dossier de Spleeter {:?}: {}", spleeter_output_subdir, e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Erreur lors du renommage/déplacement du fichier instrumental: {}", e);
                    info!("   L'instrumental brut de Spleeter se trouve peut-être ici : {:?}", spleeter_instrumental_path);
                    info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
                }
            }
        } else {
            error!("❌ Fichier instrumental ('{}') non trouvé dans le dossier de sortie de Spleeter: {:?}", instrumental_spleeter_filename, spleeter_output_subdir);
            info!("   Le fichier audio original a été conservé ici : {:?}", original_downloaded_full_path);
        }
    } else {
        error!("❌ Spleeter a échoué avec le code de sortie : {:?}. Le fichier audio original a été conservé.", spleeter_status.code());
        info!("   Chemin du fichier original : {:?}", original_downloaded_full_path);
    }

    Ok(())
}
