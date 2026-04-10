use anyhow::{Context, Result, anyhow};
use directories::BaseDirs;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const ICONIFY_SEARCH_URL: &str = "https://api.iconify.design/search";
const ICONIFY_ICON_URL: &str = "https://api.iconify.design/lucide";

pub struct LucideService {
    http: Client,
}

#[derive(Debug, Clone)]
pub struct LucideRenderSettings {
    pub color_hex: String,
    pub stroke_width: f64,
    pub size: u32,
}

impl Default for LucideRenderSettings {
    fn default() -> Self {
        Self {
            color_hex: "#ffffff".to_string(),
            stroke_width: 2.0,
            size: 24,
        }
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    icons: Vec<String>,
}

impl LucideService {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .context("Cannot create HTTP client")?;
        Ok(Self { http })
    }

    pub fn search_icons(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(default_lucide_icons());
        }

        let url = reqwest::Url::parse_with_params(
            ICONIFY_SEARCH_URL,
            &[
                ("query", trimmed),
                ("prefix", "lucide"),
                ("limit", &limit.to_string()),
            ],
        )
        .context("Cannot build Lucide search URL")?;

        let payload = self
            .http
            .get(url)
            .send()
            .context("Lucide search failed")?
            .error_for_status()
            .context("Lucide search returned an HTTP error")?
            .json::<SearchResponse>()
            .context("Invalid Lucide search response")?;

        let mut names: Vec<String> = payload
            .icons
            .into_iter()
            .filter_map(|item| item.strip_prefix("lucide:").map(|s| s.to_string()))
            .collect();

        names.sort();
        names.dedup();

        if names.is_empty() {
            return Ok(default_lucide_icons());
        }

        Ok(names)
    }

    pub fn preview_icon_svg(
        &self,
        icon_name: &str,
        settings: &LucideRenderSettings,
    ) -> Result<PathBuf> {
        self.download_or_get_icon(icon_name, settings, true)
    }

    pub fn download_icon_svg_with_settings(
        &self,
        icon_name: &str,
        settings: &LucideRenderSettings,
    ) -> Result<PathBuf> {
        self.download_or_get_icon(icon_name, settings, false)
    }

    fn download_or_get_icon(
        &self,
        icon_name: &str,
        settings: &LucideRenderSettings,
        is_preview: bool,
    ) -> Result<PathBuf> {
        let sanitized = sanitize_icon_name(icon_name)?;
        let normalized = normalize_settings(settings)?;
        let mut target_dir = icon_storage_dir()?;
        if is_preview {
            target_dir = target_dir.join("_preview");
        }
        fs::create_dir_all(&target_dir)
            .with_context(|| format!("Cannot create {}", target_dir.display()))?;

        let file_suffix = settings_suffix(&normalized);
        let target_path = target_dir.join(format!("{}-{}.svg", sanitized, file_suffix));
        if target_path.exists() {
            return Ok(target_path);
        }

        let color_value = normalized.color_hex.clone(); // Keep the # for API
        let url = reqwest::Url::parse_with_params(
            &format!("{}/{}.svg", ICONIFY_ICON_URL, sanitized),
            &[
                ("color", color_value),
                ("strokeWidth", normalized.stroke_width.to_string()),
                ("width", normalized.size.to_string()),
                ("height", normalized.size.to_string()),
            ],
        )
        .context("Cannot build Lucide icon URL")?;

        let body = self
            .http
            .get(url)
            .send()
            .with_context(|| format!("Download failed for icon {}", sanitized))?
            .error_for_status()
            .with_context(|| format!("Icon not found in Lucide: {}", sanitized))?
            .text()
            .context("Invalid SVG payload")?;

        fs::write(&target_path, body)
            .with_context(|| format!("Cannot save {}", target_path.display()))?;

        Ok(target_path)
    }
}

pub fn icon_storage_dir() -> Result<PathBuf> {
    let base = BaseDirs::new().ok_or_else(|| anyhow!("Cannot resolve user data directory"))?;
    Ok(base.data_dir().join("launcher_studio").join("icons"))
}

fn sanitize_icon_name(input: &str) -> Result<String> {
    let name = input.trim().to_lowercase();
    if name.is_empty() {
        return Err(anyhow!("Icon name cannot be empty"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(anyhow!("Icon name contains unsupported characters"));
    }
    Ok(name)
}

fn normalize_settings(input: &LucideRenderSettings) -> Result<LucideRenderSettings> {
    let stroke = input.stroke_width.clamp(0.5, 4.0);
    let size = input.size.clamp(12, 128);
    let color_hex = normalize_hex_color(&input.color_hex)?;
    Ok(LucideRenderSettings {
        color_hex,
        stroke_width: (stroke * 10.0).round() / 10.0,
        size,
    })
}

fn normalize_hex_color(input: &str) -> Result<String> {
    let mut value = input.trim().trim_start_matches('#').to_lowercase();
    if value.len() == 3 {
        value = value.chars().flat_map(|c| [c, c]).collect::<String>();
    }
    if value.len() != 6 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("Color must be a hex value like #2563eb"));
    }
    Ok(format!("#{}", value))
}

fn settings_suffix(settings: &LucideRenderSettings) -> String {
    let color = settings.color_hex.trim_start_matches('#');
    let stroke = settings.stroke_width.to_string().replace('.', "_");
    format!("c{}-sw{}-s{}", color, stroke, settings.size)
}

fn default_lucide_icons() -> Vec<String> {
    [
        "app-window",
        "badge-check",
        "bell",
        "book-open",
        "bot",
        "calendar",
        "camera",
        "circle-help",
        "cloud",
        "code",
        "folder",
        "globe",
        "heart",
        "home",
        "image",
        "layers",
        "monitor",
        "rocket",
        "search",
        "settings",
        "shield",
        "sparkles",
        "square-terminal",
        "star",
        "sun",
        "zap",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        LucideRenderSettings, normalize_hex_color, normalize_settings, sanitize_icon_name,
        settings_suffix,
    };

    #[test]
    fn sanitize_accepts_valid_lucide_name() {
        assert_eq!(sanitize_icon_name("circle-help").unwrap(), "circle-help");
    }

    #[test]
    fn sanitize_rejects_invalid_name() {
        assert!(sanitize_icon_name("../passwd").is_err());
        assert!(sanitize_icon_name("folder_open").is_err());
    }

    #[test]
    fn normalize_color_supports_short_hex() {
        assert_eq!(normalize_hex_color("#0f8").unwrap(), "#00ff88");
    }

    #[test]
    fn normalize_settings_clamps_values() {
        let settings = LucideRenderSettings {
            color_hex: "#336699".to_string(),
            stroke_width: 9.0,
            size: 1000,
        };
        let normalized = normalize_settings(&settings).unwrap();
        assert_eq!(normalized.stroke_width, 4.0);
        assert_eq!(normalized.size, 128);
    }

    #[test]
    fn settings_suffix_contains_tuned_values() {
        let settings = LucideRenderSettings {
            color_hex: "#336699".to_string(),
            stroke_width: 1.5,
            size: 24,
        };
        assert_eq!(settings_suffix(&settings), "c336699-sw1_5-s24");
    }
}
