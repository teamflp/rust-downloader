use regex::Regex;
use url::Url;
use once_cell::sync::Lazy;

/// Whitelist of allowed domains for downloads
static ALLOWED_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "vimeo.com",
    "dailymotion.com",
    "soundcloud.com",
    "bandcamp.com",
    "twitch.tv",
    "twitter.com",
    "x.com",
    "instagram.com",
    "facebook.com",
    "tiktok.com",
];

/// Regex for validating URLs
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validates a URL for download
pub fn validate_url(url_str: &str) -> Result<String, ValidationError> {
    // Check if URL is not empty
    if url_str.trim().is_empty() {
        return Err(ValidationError {
            message: "URL cannot be empty".to_string(),
        });
    }

    // Check URL format with regex
    if !URL_REGEX.is_match(url_str) {
        return Err(ValidationError {
            message: "Invalid URL format. Must start with http:// or https://".to_string(),
        });
    }

    // Parse URL
    let parsed_url = Url::parse(url_str).map_err(|e| ValidationError {
        message: format!("Failed to parse URL: {}", e),
    })?;

    // Check if scheme is http or https
    let scheme = parsed_url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(ValidationError {
            message: format!("Invalid URL scheme: {}. Only http and https are allowed", scheme),
        });
    }

    // Get domain
    let domain = parsed_url.host_str().ok_or_else(|| ValidationError {
        message: "URL must have a valid domain".to_string(),
    })?;

    // Check if domain is in whitelist
    let is_allowed = ALLOWED_DOMAINS.iter().any(|allowed| {
        domain == *allowed || domain.ends_with(&format!(".{}", allowed))
    });

    if !is_allowed {
        return Err(ValidationError {
            message: format!(
                "Domain '{}' is not allowed. Allowed domains: {}",
                domain,
                ALLOWED_DOMAINS.join(", ")
            ),
        });
    }

    // Check URL length (prevent extremely long URLs)
    if url_str.len() > 2048 {
        return Err(ValidationError {
            message: "URL is too long (max 2048 characters)".to_string(),
        });
    }

    Ok(url_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_youtube_url() {
        assert!(validate_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validate_url("https://youtu.be/dQw4w9WgXcQ").is_ok());
    }

    #[test]
    fn test_invalid_scheme() {
        assert!(validate_url("ftp://example.com/file").is_err());
    }

    #[test]
    fn test_empty_url() {
        assert!(validate_url("").is_err());
        assert!(validate_url("   ").is_err());
    }

    #[test]
    fn test_disallowed_domain() {
        assert!(validate_url("https://malicious-site.com/video").is_err());
    }

    #[test]
    fn test_url_too_long() {
        let long_url = format!("https://youtube.com/{}", "a".repeat(2100));
        assert!(validate_url(&long_url).is_err());
    }
}
