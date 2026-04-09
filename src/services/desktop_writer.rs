use crate::domain::desktop_entry::DesktopEntry;
use anyhow::{Context, Result, anyhow};
use directories::BaseDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub struct DesktopWriter;

impl DesktopWriter {
    pub fn user_applications_dir() -> Result<PathBuf> {
        if let Some(base) = BaseDirs::new() {
            let dir = base.data_dir().join("applications");
            Ok(dir)
        } else {
            Err(anyhow!("Failed to resolve XDG base directories"))
        }
    }

    pub fn write(entry: &DesktopEntry, file_name: &str, overwrite: bool) -> Result<PathBuf> {
        entry.validate().map_err(|e| anyhow!(e))?;
        let dir = Self::user_applications_dir()?;
        fs::create_dir_all(&dir).context("Creating applications directory")?;

        let sanitized = sanitize_file_name(file_name);
        let path = dir.join(format!("{}.desktop", sanitized));
        if path.exists() && !overwrite {
            return Err(anyhow!("File already exists: {}", path.display()));
        }
        let content = entry.to_ini_string();
        fs::write(&path, content).with_context(|| format!("Writing {}", path.display()))?;

        // Try to set sane permissions (not strictly required)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&path, perms)?;
        }

        refresh_desktop_database(&dir);

        Ok(path)
    }

    pub fn write_to_path(entry: &DesktopEntry, path: &Path) -> Result<PathBuf> {
        entry.validate().map_err(|e| anyhow!(e))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Creating directory {}", parent.display()))?;
        }
        if path.exists() {
            let backup_path = PathBuf::from(format!("{}.bak", path.display()));
            fs::copy(path, &backup_path)
                .with_context(|| format!("Creating backup {}", backup_path.display()))?;
        }
        let content = entry.to_ini_string();
        fs::write(path, content).with_context(|| format!("Writing {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(path, perms)?;
        }
        if let Some(parent) = path.parent() {
            refresh_desktop_database(parent);
        }
        Ok(path.to_path_buf())
    }
}

fn refresh_desktop_database(applications_dir: &Path) {
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("update-desktop-database")
            .arg(applications_dir)
            .status();
    }
}

fn sanitize_file_name(input: &str) -> String {
    let input = input.trim();
    let fallback = "desktop-entry";
    let s = if input.is_empty() { fallback } else { input };
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            _ => '-',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_file_name;

    #[test]
    fn sanitize_file_name_replaces_unsafe_chars() {
        let value = sanitize_file_name("my app?/demo");
        assert_eq!(value, "my-app--demo");
    }

    #[test]
    fn sanitize_file_name_uses_fallback_for_blank_input() {
        let value = sanitize_file_name("   ");
        assert_eq!(value, "desktop-entry");
    }
}
