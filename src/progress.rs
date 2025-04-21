/// Tente d'extraire la progression à partir d'une ligne de progression,
/// affiche la progression formatée et retourne la valeur extraite.
pub fn afficher_progression_ligne(line: &str) -> Option<(u64, u64)> {
    if let Some((current, total)) = parse_progress(line) {
        println!("{}", formater_progression(current, total));
        Some((current, total))
    } else {
        None
    }
}


/// Analyse une ligne contenant l'information de téléchargement pour en extraire
/// la progression et la taille totale en octets. Retourne `None` si la ligne
/// ne contient pas de données valides.
///
/// # Exemple
///
/// ```
/// let line = "[download]   50% of 100MiB at 1.23MiB/s ETA 00:01";
/// if let Some((current, total)) = parse_progress(line) {
///     println!("Progression : {}/{}", current, total);
/// }
/// ```
pub fn parse_progress(line: &str) -> Option<(u64, u64)> {
    if !line.contains("[download]") { return None; }
    // Cherche le pattern : pourcentage + "of" + taille
    let re = regex::Regex::new(
        r"(?x)
            \[download\] \s*     # Préfixe
            (?P<percent>\d+(?:\.\d+)?)% \s+ of \s+
            (?P<size>\d+(?:\.\d+)?)           # Valeur nombre
            (?P<unit>KiB|MiB|GiB)             # Unité
        "
    ).ok()?;

    let caps = re.captures(line)?;

    let percent = caps.name("percent")?.as_str().parse::<f64>().ok()?;
    let size = caps.name("size")?.as_str().parse::<f64>().ok()?;
    let unit = caps.name("unit")?.as_str();

    let multiplier = match unit {
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "KiB" => 1024.0,
        _ => return None,
    };

    let total_bytes = (size * multiplier) as u64;
    let downloaded_bytes = ((percent / 100.0) * size * multiplier) as u64;

    Some((downloaded_bytes, total_bytes))
}

/// Formate les informations de progression pour un affichage convivial.
/// Par exemple, on peut ici retourner une chaîne de caractère qui inclut
/// un pourcentage arrondi, ainsi qu'une barre de progression simplifiée.
///
/// # Exemple
///
/// ```
/// let (current, total) = (50_000_000, 100_000_000);
/// let message = format_progress_display(current, total);
/// println!("{}", message);
/// // Possible sortie : [#####.....] 50% (50.0MB / 100.0MB)
/// ```
pub fn formater_progression(current_bytes: u64, total_bytes: u64) -> String {
    const MB_DIVISOR: f64 = 1024.0 * 1024.0;
    const LONGUEUR_BARRE: usize = 10;

    let pourcentage = if total_bytes > 0 {
        (current_bytes as f64 / total_bytes as f64 * 100.0).round() as u64
    } else {
        0
    };

    let octets_actuels_mb = current_bytes as f64 / MB_DIVISOR;
    let octets_totaux_mb = total_bytes as f64 / MB_DIVISOR;

    let longueur_remplie = (pourcentage as usize * LONGUEUR_BARRE) / 100;
    let barre = format!(
        "[{}{}]",
        "#".repeat(longueur_remplie),
        ".".repeat(LONGUEUR_BARRE - longueur_remplie)
    );

    format!(
        "{} {}% ({:.1}MB / {:.1}MB)",
        barre, pourcentage, octets_actuels_mb, octets_totaux_mb
    )
}
