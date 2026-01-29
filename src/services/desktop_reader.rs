use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::desktop_entry::DesktopEntry;

pub struct DesktopReader;

impl DesktopReader {
    pub fn user_applications_dir() -> Option<PathBuf> {
        BaseDirs::new().map(|b: BaseDirs| b.home_dir().join(".local/share/applications"))
    }

    pub fn list_desktop_files() -> Result<Vec<PathBuf>> {
        let dir: PathBuf = match Self::user_applications_dir() {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        let mut files = Vec::new();
        if dir.exists() {
            for entry in fs::read_dir(&dir).with_context(|| format!("Reading {}", dir.display()))? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    pub fn read_from_path(path: &Path) -> Result<DesktopEntry> {
        let content = fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Ok(parse_desktop_content(&content))
    }
}

fn parse_desktop_content(content: &str) -> DesktopEntry {
    let mut entry = DesktopEntry::default();
    let mut in_desktop = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') { continue; }
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop { continue; }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim();
            let val = v.trim().to_string();
            match key {
                "Type" => entry.type_field = val,
                "Name" => entry.name = val,
                _ if key.starts_with("Name[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("Name[").trim_end_matches("]").to_string();
                    entry.name_localized.push((lang, val));
                }
                "GenericName" => entry.generic_name = Some(val),
                _ if key.starts_with("GenericName[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("GenericName[").trim_end_matches("]").to_string();
                    entry.generic_name_localized.push((lang, val));
                }
                "Comment" => entry.comment = Some(val),
                _ if key.starts_with("Comment[") && key.ends_with("]") => {
                    let lang = key.trim_start_matches("Comment[").trim_end_matches("]").to_string();
                    entry.comment_localized.push((lang, val));
                }
                "Exec" => entry.exec = val,
                "TryExec" => entry.try_exec = Some(val),
                "Icon" => entry.icon = Some(val),
                "Path" => entry.path = Some(val),
                "URL" => entry.url = Some(val),
                "Terminal" => entry.terminal = val.eq_ignore_ascii_case("true"),
                "NoDisplay" => entry.no_display = val.eq_ignore_ascii_case("true"),
                "StartupNotify" => entry.startup_notify = val.eq_ignore_ascii_case("true"),
                "Categories" => entry.categories = split_semicolon(&val),
                "MimeType" => entry.mime_type = split_semicolon(&val),
                "Keywords" => entry.keywords = split_semicolon(&val),
                "OnlyShowIn" => entry.only_show_in = split_semicolon(&val),
                "NotShowIn" => entry.not_show_in = split_semicolon(&val),
                "Actions" => entry.actions = split_semicolon(&val),
                _ => entry.extra.push((key.to_string(), val)),
            }
        }
    }
    if entry.type_field.is_empty() { entry.type_field = "Application".into(); }
    entry
}

fn split_semicolon(s: &str) -> Vec<String> {
    s.split(';').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}
