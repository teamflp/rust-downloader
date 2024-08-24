// Fonction pour analyser la progression du téléchargement et la taille totale
pub fn parse_progress(line: &str) -> Option<(u64, u64)> {
    if line.contains("[download]") {
        if let Some(start) = line.find('[') {
            let segments: Vec<&str> = line[start..].split_whitespace().collect();
            if segments.len() > 4 {
                let progress_str = segments[1].trim_end_matches('%');
                let total_size_str = segments[3].replace(",", "");
                let multiplier = if total_size_str.contains("GiB") {
                    1024.0 * 1024.0 * 1024.0
                } else if total_size_str.contains("MiB") {
                    1024.0 * 1024.0
                } else if total_size_str.contains("KiB") {
                    1024.0
                } else {
                    1.0
                };
                let total_size_str = total_size_str.replace("GiB", "").replace("MiB", "").replace("KiB", "");

                if let (Ok(progress), Ok(total_size)) = (progress_str.parse::<f64>(), total_size_str.parse::<f64>()) {
                    return Some(((progress / 100.0 * total_size * multiplier) as u64, (total_size * multiplier) as u64));
                }
            }
        }
    }
    None
}
