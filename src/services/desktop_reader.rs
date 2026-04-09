use anyhow::{Context, Result};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::desktop_entry::DesktopEntry;

pub struct DesktopReader;

impl DesktopReader {
    pub fn user_applications_dir() -> Option<PathBuf> {
        BaseDirs::new().map(|b: BaseDirs| b.data_dir().join("applications"))
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
        let content =
            fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Ok(DesktopEntry::from_ini_string(&content))
    }
}
