use std::path::Path;
use std::process::Stdio;
use std::process::{Command, exit};
use std::env;
use colored::*;
use std::io::{self, Write};

/// Vérifie si une commande système est disponible dans le PATH
fn is_command_available(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Installe Homebrew (macOS) s'il est absent
fn install_brew() -> bool {
    println!("⚙️ Homebrew n'est pas installé. Installation en cours...");

    let cmd = r#"/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)""#;

    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::inherit()) // Show installation output
        .stderr(Stdio::inherit()) // Show installation error output
        .status()
        .expect("Erreur lors de l'exécution du script d'installation de Homebrew.");

    if !status.success() {
        eprintln!("{}", "❌ L'installation de Homebrew a échoué.".red());
        eprintln!("Veuillez installer Homebrew manuellement et réessayer.");
        return false;
    }

    println!("{}", "✅ Homebrew installé avec succès !".green());
    println!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal ou de sourcer votre fichier de profil (ex: ~/.zshrc, ~/.bash_profile) pour que brew soit pleinement utilisable.".yellow());
    true
}

/// Installe Chocolatey (Windows) s'il est absent
fn install_chocolatey() -> bool {
    println!("⚙️ Chocolatey n'est pas installé. Installation en cours...");

    let cmd = r#"Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"#;

    let status = Command::new("powershell")
        .args(&["-NoProfile", "-InputFormat", "None", "-ExecutionPolicy", "Bypass", "-Command", cmd])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Erreur lors de l'exécution du script d'installation de Chocolatey.");

    if !status.success() {
        eprintln!("{}", "❌ L'installation de Chocolatey a échoué.".red());
        eprintln!("Veuillez installer Chocolatey manuellement et réessayer.");
        return false;
    }

    println!("{}", "✅ Chocolatey installé avec succès !".green());
    println!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal pour que choco soit pleinement utilisable.".yellow());
    true
}

/// Tentative d'installation via apt (Linux)
fn install_apt(package: &str) -> bool {
    let (command_name, base_args) = if env::var("USER").unwrap_or_default() == "root" {
        ("apt", Vec::new())
    } else if is_command_available("sudo") {
        ("sudo", vec!["apt"])
    } else {
        eprintln!("{}", "❌ La commande 'sudo' est nécessaire pour 'apt' mais n'est pas trouvée et l'utilisateur n'est pas root.".red());
        eprintln!("Veuillez installer le paquet '{}' manuellement ou exécuter en tant que root.", package);
        return false;
    };

    println!("⚙️ Mise à jour des dépôts apt (peut nécessiter un mot de passe)...");
    let mut update_args = base_args.clone();
    update_args.extend_from_slice(&["update", "-y"]);

    let update_status = Command::new(command_name)
        .args(&update_args)
        .status()
        .expect("Erreur lors de la mise à jour des dépôts apt.");

    if !update_status.success() {
        eprintln!("{}", "⚠️ Échec de la mise à jour des dépôts apt (apt update). Tentative d'installation quand même...".yellow());
    }

    println!("⚙️ Tentative d'installation de '{}' avec apt (peut nécessiter un mot de passe)...", package);
    let mut install_args = base_args;
    install_args.extend_from_slice(&["install", "-y", package]);

    let status = Command::new(command_name)
        .args(&install_args)
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec apt.", package));

    if !status.success() {
        eprintln!("❌ L'installation de {} avec apt a échoué.", package.red());
        return false;
    }
    println!("✅ {} installé avec succès via apt!", package.green());
    true
}

/// Tentative d'installation via brew (macOS)
fn install_brew_package(package: &str) -> bool {
    if !is_command_available("brew") {
        if !install_brew() {
            return false;
        }
    }
    println!("⚙️ Tentative d'installation de '{}' avec brew...", package);
    let status = Command::new("brew")
        .args(&["install", package])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'exécution de brew install {}.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} avec brew a échoué.", package.red());
        return false;
    }
    println!("✅ {} installé avec succès via brew!", package.green());
    true
}

/// Tentative d'installation via choco (Windows)
fn install_choco_package(package: &str) -> bool {
    if !is_command_available("choco") {
        if !install_chocolatey() {
            return false;
        }
    }
    println!("⚙️ Tentative d'installation de '{}' avec choco...", package);
    let status = Command::new("choco")
        .args(&["install", package, "-y"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'exécution de choco install {}.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} avec choco a échoué.", package.red());
        return false;
    }
    println!("✅ {} installé avec succès via choco!", package.green());
    true
}

/// Tentative d'installation via scoop (Windows)
fn install_scoop_package(package: &str) -> bool {
    if !is_command_available("scoop") {
        println!("⚙️ Scoop n'est pas installé. Tentative d'installation...");
        let cmd = "Set-ExecutionPolicy RemoteSigned -Scope CurrentUser; irm get.scoop.sh | iex";
        let status = Command::new("powershell")
            .args(&["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Erreur lors de l'installation de Scoop.");

        if !status.success() {
            eprintln!("{}", "❌ L'installation de Scoop a échoué.".red());
            return false;
        }
        println!("{}", "✅ Scoop installé avec succès!".green());
        println!("{}", "NOTE: Vous pourriez avoir besoin de redémarrer votre terminal pour que scoop soit pleinement utilisable.".yellow());
    }
    println!("⚙️ Tentative d'installation de '{}' avec scoop...", package);
    let status = Command::new("scoop")
        .args(&["install", package])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect(&format!("Erreur lors de l'installation de {} avec scoop.", package));
    if !status.success() {
        eprintln!("❌ L'installation de {} avec scoop a échoué.", package.red());
        return false;
    }
    println!("✅ {} installé avec succès via scoop!", package.green());
    true
}

/// Installe ffmpeg de manière multiplateforme
pub fn install_ffmpeg() {
    println!("⚙️ Installation de ffmpeg...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => install_apt("ffmpeg"),
        "macos" => install_brew_package("ffmpeg"),
        "windows" => install_choco_package("ffmpeg") || install_scoop_package("ffmpeg"),
        _ => {
            eprintln!("❌ Système d'exploitation '{}' non supporté pour l'installation automatique de ffmpeg.", os.red());
            false
        }
    };

    if !success {
        eprintln!("{}", "❌ L'installation de ffmpeg a échoué. Veuillez l'installer manuellement.".red());
        exit(1);
    }
    println!("{}", "✅ ffmpeg est maintenant prêt.".green());
}

/// Helper function for installing yt-dlp on Linux
fn install_yt_dlp_linux_internal() -> bool {
    let (sudo_prefix, _is_root) = if env::var("USER").unwrap_or_default() == "root" {
        ("", true)
    } else if is_command_available("sudo") {
        ("sudo ", false)
    } else {
        eprintln!("{}", "❌ 'sudo' est requis pour installer yt-dlp globalement mais n'est pas trouvé.".red());
        return false;
    };

    let install_dir = "/usr/local/bin";
    let yt_dlp_path = format!("{}/yt-dlp", install_dir);

    println!("Téléchargement de yt-dlp vers {}...", yt_dlp_path);
    let cmd_dl = format!(
        "{}curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o {}",
        sudo_prefix, yt_dlp_path
    );
    let status_dl = Command::new("sh").arg("-c").arg(&cmd_dl).status();

    if !status_dl.map_or(false, |s| s.success()) {
        eprintln!("{}", "❌ Échec du téléchargement de yt-dlp.".red());
        return false;
    }

    println!("Configuration des permissions pour {}...", yt_dlp_path);
    let cmd_chmod = format!("{}chmod a+rx {}", sudo_prefix, yt_dlp_path);
    let status_chmod = Command::new("sh").arg("-c").arg(&cmd_chmod).status();

    if !status_chmod.map_or(false, |s| s.success()) {
        eprintln!("{}", "❌ Échec de la configuration des permissions pour yt-dlp.".red());
        if sudo_prefix == "sudo " { // Attempt to clean up only if sudo was used for download
            let _ = Command::new("sh").arg("-c").arg(format!("{}rm -f {}", sudo_prefix, yt_dlp_path)).status();
        }
        return false;
    }
    println!("{}", "✅ yt-dlp installé avec succès via curl!".green());
    true
}


/// Installe yt-dlp de manière multiplateforme
pub fn install_yt_dlp() {
    println!("⚙️ Installation de yt-dlp...");
    let os = env::consts::OS;
    let success = match os {
        "linux" => install_yt_dlp_linux_internal(),
        "macos" => install_brew_package("yt-dlp"),
        "windows" => install_choco_package("yt-dlp") || install_scoop_package("yt-dlp"),
        _ => {
            eprintln!("❌ Système d'exploitation '{}' non supporté pour l'installation automatique de yt-dlp.", os.red());
            false
        }
    };

    if !success {
        eprintln!("{}", "❌ L'installation de yt-dlp a échoué. Veuillez l'installer manuellement.".red());
        exit(1);
    }
    println!("{}", "✅ yt-dlp est maintenant prêt.".green());
}

/// Tente d'installer Spleeter et ses dépendances Python.
fn do_install_spleeter_os_specific() -> bool {
    let os = env::consts::OS;
    let mut pip_ready = false;

    // Déterminer l'interpréteur Python à utiliser
    let mut initial_python_cmd_str = "python".to_string();
    if os == "macos" {
        let homebrew_python3_path = Path::new("/usr/local/bin/python3");
        if homebrew_python3_path.exists() && is_command_available(homebrew_python3_path.to_str().unwrap_or_default()) {
            initial_python_cmd_str = homebrew_python3_path.to_str().unwrap().to_string();
        } else if is_command_available("python3") {
            initial_python_cmd_str = "python3".to_string();
        } else if is_command_available("python") {
            // Reste "python"
        } else {
            eprintln!("{}", "❌ Aucun interpréteur Python (python3 ou python) n'a été trouvé.".red());
            return false;
        }
    } else { // Pour Linux, Windows, etc.
        if is_command_available("python3") {
            initial_python_cmd_str = "python3".to_string();
        } else if is_command_available("python") {
            // Reste "python"
        } else {
            eprintln!("{}", "❌ Aucun interpréteur Python (python3 ou python) n'a été trouvé.".red());
            return false;
        }
    }
    let python_cmd_for_check = &initial_python_cmd_str;

    // Vérifier si pip est fonctionnel avec l'interpréteur choisi
    if Command::new(python_cmd_for_check).args(&["-m", "pip", "--version"]).status().map_or(false, |s| s.success()) {
        pip_ready = true;
    } else {
        println!("🐍 Pip (gestionnaire de paquets Python) n'est pas trouvé ou n'est pas fonctionnel avec '{}'.", python_cmd_for_check);
        match os {
            "linux" => {
                println!("Tentative d'installation de python3-pip via apt...");
                if install_apt("python3-pip") { // install_apt devrait utiliser python3
                    pip_ready = Command::new("python3").args(&["-m", "pip", "--version"]).status().map_or(false, |s| s.success());
                    if pip_ready { initial_python_cmd_str = "python3".to_string(); }
                }
            }
            "macos" => {
                println!("Tentative d'installation de python via Homebrew (qui inclut pip)...");
                if install_brew_package("python") { // Ceci installe python3 et pip3
                    let homebrew_python3_path_str = "/usr/local/bin/python3";
                    if Path::new(homebrew_python3_path_str).exists() && is_command_available(homebrew_python3_path_str) {
                        initial_python_cmd_str = homebrew_python3_path_str.to_string();
                        pip_ready = Command::new(homebrew_python3_path_str).args(&["-m", "pip", "--version"]).status().map_or(false, |s| s.success());
                    } else if is_command_available("python3") { // Fallback
                        initial_python_cmd_str = "python3".to_string();
                        pip_ready = Command::new("python3").args(&["-m", "pip", "--version"]).status().map_or(false, |s| s.success());
                    }
                }
            }
            "windows" => {
                println!("Tentative d'installation de python via Chocolatey (qui inclut pip)...");
                if install_choco_package("python") {
                    pip_ready = Command::new("python").args(&["-m", "pip", "--version"]).status().map_or(false, |s| s.success());
                    if pip_ready { initial_python_cmd_str = "python".to_string(); }
                }
            }
            _ => {
                eprintln!("{}", "❌ Système d'exploitation non supporté pour l'installation automatique de Python/pip.".red());
            }
        }
    }

    if !pip_ready {
        eprintln!("{}", "❌ Pip n'a pas pu être installé ou rendu fonctionnel. Spleeter ne peut pas être installé automatiquement.".red());
        return false;
    }

    let final_python_cmd = &initial_python_cmd_str;

    // --- Tentative de mise à jour de pip ---
    println!("🐍 Pip est disponible avec '{}'. Tentative de mise à jour de pip...", final_python_cmd);
    let mut pip_upgrade_args = vec!["-m", "pip", "install"];
    if os == "macos" {
        pip_upgrade_args.push("--break-system-packages");
        pip_upgrade_args.push("--user");
    }
    pip_upgrade_args.push("--upgrade");
    pip_upgrade_args.push("pip");

    let pip_upgrade_output = Command::new(final_python_cmd)
        .args(&pip_upgrade_args)
        .output()
        .expect("Erreur lors de l'exécution de la mise à jour de pip.");

    if !pip_upgrade_output.status.success() {
        println!("{}", "⚠️  La mise à jour de pip a échoué ou n'était pas nécessaire. Continuation...".yellow());
    } else {
        println!("{}", "✅ Pip mis à jour avec succès.".green());
    }

    // --- Tentative d'installation de Spleeter ---
    println!("⚙️  Installation de Spleeter via pip avec '{}' (cela peut prendre plusieurs minutes à cause de TensorFlow)...", final_python_cmd);
    let mut spleeter_install_args = vec!["-m", "pip", "install"];
    if os == "macos" {
        spleeter_install_args.push("--break-system-packages");
        spleeter_install_args.push("--user");
    }
    spleeter_install_args.push("spleeter");

    let spleeter_install_output = Command::new(final_python_cmd)
        .args(&spleeter_install_args)
        .output()
        .expect("Erreur lors de l'exécution de pip install spleeter.");

    if !spleeter_install_output.status.success() {
        eprintln!("{}", "❌ L'installation de Spleeter via pip a échoué.".red());

        eprintln!("{}", "   Une erreur technique est survenue lors de la tentative d'installation de Spleeter avec pip.".yellow());
        eprintln!("{}", "   Cela est souvent dû à des incompatibilités avec votre version actuelle de Python ou des problèmes avec les dépendances de Spleeter (comme numpy ou TensorFlow).".yellow());

        let spleeter_stderr_str = String::from_utf8_lossy(&spleeter_install_output.stderr);

        eprintln!("   Vérifiez votre connexion internet.");
        eprintln!("   Causes possibles : Incompatibilité de version Python (Spleeter/TensorFlow avec Python >3.11), PEP 668 (environnement externe), conflits de dépendances, permissions.");

        if os == "macos" && spleeter_stderr_str.contains("externally-managed-environment") {
            eprintln!("   L'option '--break-system-packages --user' a été tentée pour macOS pour gérer l'erreur 'externally-managed-environment'.");
        }

        if spleeter_stderr_str.contains("NameError: name 'CCompiler' is not defined") || (spleeter_stderr_str.contains("metadata-generation-failed") && spleeter_stderr_str.contains("numpy")) {
            eprintln!("   L'erreur semble liée à la compilation de 'numpy', une dépendance de Spleeter. Cela se produit fréquemment avec des versions de Python trop récentes (comme 3.12+).");
        } else {
            eprintln!("   Si l'erreur concerne la compilation d'un paquet comme 'numpy', cela est souvent dû à une version de Python trop récente (comme 3.12+).");
        }
        eprintln!("   Spleeter et ses dépendances (notamment TensorFlow) sont plus stables avec Python 3.7-3.11.");

        eprintln!("   Vous pouvez essayer d'exécuter cette commande manuellement pour plus de détails (cela affichera la sortie technique complète) :");
        let mut manual_cmd_display_parts = vec![final_python_cmd.to_string(), "-m".to_string(), "pip".to_string(), "install".to_string()];
        if os == "macos" {
            manual_cmd_display_parts.push("--break-system-packages".to_string());
            manual_cmd_display_parts.push("--user".to_string());
        }
        manual_cmd_display_parts.push("spleeter".to_string());
        eprintln!("   {}", manual_cmd_display_parts.join(" "));
        return false;
    }

    println!("{}", "✅ Spleeter semble avoir été installé avec succès via pip.".green());
    println!("{}", "NOTE: Si la commande 'spleeter' n'est pas immédiatement trouvée,".yellow());
    println!("{}", "      assurez-vous que le répertoire des scripts Python est dans votre PATH.".yellow());
    if os == "macos" {
        println!("{}", "      Pour les installations '--user' sur macOS, cela peut être '~/Library/Python/X.Y/bin' (remplacez X.Y par votre version Python).".yellow());
    } else {
        println!("{}", "      (Ex: ~/.local/bin sur Linux, ou %APPDATA%\\Python\\PythonXX\\Scripts sur Windows)".yellow());
    }
    println!("{}", "      Vous pourriez avoir besoin de redémarrer votre terminal ou votre session.".yellow());
    true
}

/// Demande à l'utilisateur s'il souhaite continuer sans Spleeter après un échec d'installation.
fn demander_continuer_sans_spleeter() -> bool {
    loop {
        print!("{}", "\n🤔 L'installation automatique de Spleeter a échoué. ".yellow());
        print!("{}", "L'extraction instrumentale ne sera pas disponible.".yellow());
        print!("{}", "\nSouhaitez-vous continuer à utiliser l'application pour les téléchargements de vidéos et d'audios (sans cette fonctionnalité) ? (o/N) : ".bold());
        io::stdout().flush().unwrap_or_else(|e| eprintln!("Erreur lors du flush stdout: {}", e));

        let mut reponse = String::new();
        if io::stdin().read_line(&mut reponse).is_err() {
            println!("{}", "❌ Erreur de lecture de votre réponse. Veuillez réessayer.".red());
            continue;
        }
        let reponse = reponse.trim().to_lowercase();

        if reponse == "o" {
            println!("{}", "✅ D'accord, continuation sans l'extraction instrumentale.".green());
            return true;
        } else if reponse == "n" || reponse.is_empty() { // Default to No
            println!("{}", "\n👋 Merci d'avoir utilisé Panther Downloader. À bientôt !\n".blue().bold());
            return false;
        } else {
            println!("{}", "❌ Réponse invalide. Veuillez entrer 'o' pour oui ou 'n' pour non.".red());
        }
    }
}


/// Installe Spleeter si nécessaire.
pub fn install_spleeter() {
    println!("⚙️  Vérification et installation de Spleeter...");
    if !do_install_spleeter_os_specific() { // Si l'installation automatique échoue
        eprintln!("{}", "❌ L'installation automatique de Spleeter a échoué.".red());
        eprintln!("{}", "----------------------------------------------------------------------".yellow());
        eprintln!("{}", "IMPORTANT: Problème de compatibilité Python détecté pour Spleeter.".bold().red());
        eprintln!("{}", "----------------------------------------------------------------------".yellow());
        eprintln!("Spleeter et ses dépendances (comme TensorFlow et une ancienne version de numpy)");
        eprintln!("sont souvent {} avec les versions très récentes de Python (ex: Python 3.12+).", "incompatibles".bold().red());
        eprintln!("Votre version actuelle de Python semble causer des échecs lors de la compilation de ces dépendances.");

        let os_type = env::consts::OS;
        if os_type == "macos" {
            eprintln!("\nPour résoudre cela sur votre Mac, la {} est d'utiliser un environnement Python dédié avec une version compatible (Python 3.9, 3.10, ou 3.11 sont recommandés).", "SOLUTION LA PLUS FIABLE".bold().green());
            eprintln!("\n{} ÉTAPES DÉTAILLÉES POUR MAC (RECOMMANDÉ) :", "👉".green());
            eprintln!("   -------------------------------------------------------------------");
            eprintln!("   {} Objectif : Créer un environnement Python isolé avec Python 3.9 (par exemple) pour Spleeter.", "🎯".cyan());
            eprintln!("   -------------------------------------------------------------------");
            eprintln!("\n   1. {}Ouvrez une nouvelle fenêtre de Terminal{}.", "`".bold(), "`".bold());
            eprintln!("\n   2. {}Installez une version compatible de Python via Homebrew (si pas déjà fait) :{}", "🐍".yellow(), "(Exemple avec Python 3.9)".italic());
            eprintln!("      {} brew install python@3.9", "$".dimmed());
            eprintln!("      {} (Cela rendra `python3.9` disponible)", "ℹ️".dimmed());
            eprintln!("\n   3. {}Créez un environnement virtuel pour Spleeter avec cette version de Python :", "🛠️".yellow());
            eprintln!("      {} python3.9 -m venv ~/spleeter_env", "$".dimmed());
            eprintln!("      {} (Ceci crée un dossier `spleeter_env` dans votre répertoire personnel)", "ℹ️".dimmed());
            eprintln!("\n   4. {}Activez cet environnement virtuel :", "💡".yellow());
            eprintln!("      {} source ~/spleeter_env/bin/activate", "$".dimmed());
            eprintln!("      {} (Votre invite de commande devrait maintenant commencer par `(spleeter_env)`)", "ℹ️".dimmed());
            eprintln!("\n   5. {}Installez Spleeter DANS cet environnement activé :", "📦".yellow());
            eprintln!("      {} pip install spleeter", "$".dimmed());
            eprintln!("      {} (Cette installation a de fortes chances de réussir ici)", "ℹ️".dimmed());
            eprintln!("\n   6. {}Utilisation future de Spleeter :", "🚀".yellow());
            eprintln!("      {} Chaque fois que vous voudrez utiliser Spleeter (y compris avec cet outil),", "ℹ️".dimmed());
            eprintln!("      {} vous devrez d'abord activer l'environnement : `source ~/spleeter_env/bin/activate`", "ℹ️".dimmed());
            eprintln!("      {} Une fois Spleeter installé dans cet environnement, cet outil devrait pouvoir le trouver si l'environnement est actif.", "ℹ️".dimmed());
            eprintln!("\n   7. {}Pour quitter l'environnement virtuel (quand vous avez fini) :", "🚪".yellow());
            eprintln!("      {} deactivate", "$".dimmed());
            eprintln!("\n{} AUTRE OPTION (moins recommandée, peut toujours échouer avec Python 3.12+) :", "⚠️".yellow());
            eprintln!("   Si vous souhaitez toujours essayer avec votre Python global (actuellement {}) et que l'erreur PEP 668 était le problème principal :", env::var("PYTHON_VERSION").unwrap_or_else(|_| "inconnue".to_string()).italic());
            eprintln!("     {} pip install --user spleeter", "$".dimmed());
            eprintln!("     (Cela nécessite que {} soit dans votre PATH)", "~/Library/Python/X.Y/bin".italic());
        } else {
            // Instructions génériques pour les autres OS
            eprintln!("\nIl est fortement recommandé d'utiliser un {} avec une version de Python compatible (Python 3.7-3.11).", "environnement virtuel Python".bold());
            eprintln!("Veuillez consulter la documentation de Python pour créer un environnement virtuel sur votre système.");
            eprintln!("Une fois l'environnement activé, exécutez : {} pip install spleeter", "$".dimmed());
        }

        eprintln!("\nConsultez la documentation officielle de Spleeter et TensorFlow pour les compatibilités de version.");

        // Demander à l'utilisateur s'il veut continuer sans Spleeter
        if !demander_continuer_sans_spleeter() {
            exit(1); // Quitter si l'utilisateur ne veut pas continuer
        }
        // Si l'utilisateur veut continuer, la fonction se termine ici.
        // Spleeter n'est pas installé, mais l'application ne quitte pas.
    }
}

/// Vérifie et installe tous les outils nécessaires
pub fn ensure_dependencies() {
    println!("{}", "🔍 Vérification des dépendances...".bold());
    let mut all_core_deps_ready = true;

    if !is_command_available("ffmpeg") {
        install_ffmpeg(); // Cette fonction appelle exit(1) en cas d'échec
        if !is_command_available("ffmpeg") { // Ne devrait pas être atteint si install_ffmpeg quitte
            all_core_deps_ready = false;
        }
    } else {
        println!("{}", "✅ ffmpeg est déjà installé.".green());
    }

    if !is_command_available("yt-dlp") {
        install_yt_dlp(); // Cette fonction appelle exit(1) en cas d'échec
        if !is_command_available("yt-dlp") { // Ne devrait pas être atteint
            all_core_deps_ready = false;
        }
    } else {
        println!("{}", "✅ yt-dlp est déjà installé.".green());
    }

    let spleeter_initially_available = is_command_available("spleeter");
    if !spleeter_initially_available {
        install_spleeter(); // Gère maintenant la sortie si l'utilisateur ne veut pas continuer
    } else {
        println!("{}", "✅ Spleeter est déjà installé.".green());
    }

    // Message final
    if all_core_deps_ready {
        if is_command_available("spleeter") {
            println!("{}", "🎉 Toutes les dépendances (y compris Spleeter) sont prêtes !".green().bold());
        } else {
            println!("{}", "✅ Les dépendances de base (ffmpeg, yt-dlp) sont prêtes.".green());
            println!("{}", "⚠️ Spleeter n'a pas pu être installé ou n'est pas disponible. L'extraction instrumentale ne fonctionnera pas.".yellow());
            println!("{}", "   L'application continuera pour les autres téléchargements.".yellow());
        }
    } else {
        // Ce cas ne devrait pas être atteint si install_ffmpeg/yt-dlp quittent correctement en cas d'échec.
        println!("{}", "❌ Certaines dépendances de base n'ont pas pu être installées. L'application ne peut pas continuer.".red());
        exit(1);
    }
}