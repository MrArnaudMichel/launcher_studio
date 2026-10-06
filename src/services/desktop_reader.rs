use anyhow::{Context, Result};
use directories::BaseDirs;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::desktop_entry::DesktopEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesktopLocation {
    #[default]
    All,
    User,    // ~/.local/share/applications or custom user paths
    System,  // /usr/share/applications, /usr/local/share/applications, /run/host/usr, /opt, /etc
    Flatpak, // /var/lib/flatpak, ~/.local/share/flatpak, /run/host/...flatpak
}

impl DesktopLocation {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::User => "Home",
            Self::System => "System",
            Self::Flatpak => "Flatpak",
        }
    }

    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::User => "Home",
            Self::System => "/usr",
            Self::Flatpak => "Flatpak",
        }
    }
}

pub struct DesktopReader;

impl DesktopReader {
    /// Returns the user applications directory ($XDG_DATA_HOME/applications or ~/.local/share/applications).
    pub fn user_applications_dir() -> Option<PathBuf> {
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            let trimmed = xdg_data_home.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed).join("applications"));
            }
        }
        BaseDirs::new().map(|b: BaseDirs| b.data_dir().join("applications"))
    }

    /// Returns a list of all existing system applications directories.
    /// It checks XDG_DATA_DIRS, standard host paths, and Flatpak sandbox paths.
    pub fn system_applications_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        let mut seen = HashSet::new();
        let mut candidates = Vec::new();

        // 1. From XDG_DATA_DIRS environment variable
        if let Ok(xdg_dirs) = env::var("XDG_DATA_DIRS") {
            for part in xdg_dirs.split(':') {
                let p = PathBuf::from(part.trim());
                if !p.as_os_str().is_empty() {
                    if p.ends_with("applications") {
                        candidates.push(p);
                    } else {
                        candidates.push(p.join("applications"));
                    }
                }
            }
        }

        // 2. Standard system locations on Linux
        candidates.push(PathBuf::from("/usr/share/applications"));
        candidates.push(PathBuf::from("/usr/local/share/applications"));
        candidates.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        // 3. Flatpak host system locations (when running inside Flatpak sandbox)
        candidates.push(PathBuf::from("/run/host/usr/share/applications"));
        candidates.push(PathBuf::from("/run/host/usr/local/share/applications"));
        candidates.push(PathBuf::from(
            "/run/host/var/lib/flatpak/exports/share/applications",
        ));
        candidates.push(PathBuf::from("/run/host/user-share/applications"));

        // Flatpak exports in user data dir
        if let Some(base) = BaseDirs::new() {
            candidates.push(base.data_dir().join("flatpak/exports/share/applications"));
        }

        for path in candidates {
            if path.is_dir() {
                let norm = path.canonicalize().unwrap_or_else(|_| path.clone());
                if seen.insert(norm) {
                    dirs.push(path);
                }
            }
        }

        dirs
    }

    /// Returns the primary system applications directory that exists.
    pub fn primary_system_applications_dir() -> Option<PathBuf> {
        let sys_dirs = Self::system_applications_dirs();
        if let Some(host_p) = sys_dirs.iter().find(|p| p.starts_with("/run/host")) {
            return Some(host_p.clone());
        }
        if let Some(usr_p) = sys_dirs
            .iter()
            .find(|p| p == &Path::new("/usr/share/applications"))
        {
            return Some(usr_p.clone());
        }
        sys_dirs.into_iter().next()
    }

    /// Checks whether a given path belongs to a system (read-only) directory
    /// or is located outside the user applications directory.
    pub fn is_system_path(path: &Path) -> bool {
        if let Some(user_dir) = Self::user_applications_dir()
            && path.starts_with(&user_dir)
        {
            return false;
        }
        let is_in_system = Self::system_applications_dirs()
            .iter()
            .any(|dir| path.starts_with(dir));
        if is_in_system {
            return true;
        }

        path.starts_with("/usr")
            || path.starts_with("/run/host")
            || path.starts_with("/var/lib/flatpak")
            || path.starts_with("/etc")
            || path.starts_with("/opt")
    }

    /// Detects the location category of a given desktop file path.
    pub fn detect_location(path: &Path) -> DesktopLocation {
        let s = path.to_string_lossy();
        if s.contains("flatpak") {
            return DesktopLocation::Flatpak;
        }
        if let Some(user_dir) = Self::user_applications_dir()
            && path.starts_with(&user_dir)
        {
            return DesktopLocation::User;
        }
        if s.starts_with("/usr")
            || s.starts_with("/run/host/usr")
            || s.starts_with("/opt")
            || s.starts_with("/etc")
        {
            return DesktopLocation::System;
        }
        if Self::is_system_path(path) {
            return DesktopLocation::System;
        }
        DesktopLocation::User
    }

    /// Lists all .desktop files from user and system application directories.
    /// User .desktop files take precedence over system ones with identical filenames.
    pub fn list_desktop_files() -> Result<Vec<PathBuf>> {
        let user_dir = Self::user_applications_dir();
        let sys_dirs = Self::system_applications_dirs();
        Ok(Self::list_desktop_files_from_dirs(
            user_dir.as_deref(),
            &sys_dirs,
        ))
    }

    /// Helper function to list desktop files given explicit user dir and system dirs.
    pub fn list_desktop_files_from_dirs(
        user_dir: Option<&Path>,
        system_dirs: &[PathBuf],
    ) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut seen_filenames = HashSet::new();

        // 1. First scan user applications directory (highest precedence)
        if let Some(udir) = user_dir
            && udir.is_dir()
            && let Ok(entries) = fs::read_dir(udir)
        {
            let mut user_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "desktop").unwrap_or(false) {
                    if let Some(file_name) = path.file_name() {
                        seen_filenames.insert(file_name.to_os_string());
                    }
                    user_files.push(path);
                }
            }
            user_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            files.extend(user_files);
        }

        // 2. Scan system applications directories
        let mut system_files = Vec::new();
        for dir in system_dirs {
            if dir.is_dir()
                && let Ok(entries) = fs::read_dir(dir)
            {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().map(|e| e == "desktop").unwrap_or(false)
                        && let Some(file_name) = path.file_name()
                        && seen_filenames.insert(file_name.to_os_string())
                    {
                        system_files.push(path);
                    }
                }
            }
        }

        system_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files.extend(system_files);

        files
    }

    pub fn read_from_path(path: &Path) -> Result<DesktopEntry> {
        let content =
            fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))?;
        Ok(DesktopEntry::from_ini_string(&content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_list_desktop_files_priority_and_deduplication() {
        let temp_base = std::env::temp_dir().join(format!(
            "ls_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let user_dir = temp_base.join("user_apps");
        let sys_dir1 = temp_base.join("sys_apps_1");
        let sys_dir2 = temp_base.join("sys_apps_2");

        fs::create_dir_all(&user_dir).unwrap();
        fs::create_dir_all(&sys_dir1).unwrap();
        fs::create_dir_all(&sys_dir2).unwrap();

        // Create files in user_dir:
        // app_common.desktop, user_only.desktop, ignored.txt
        let user_common = user_dir.join("app_common.desktop");
        let mut f = File::create(&user_common).unwrap();
        writeln!(f, "[Desktop Entry]\nName=User Common").unwrap();

        let user_only = user_dir.join("user_only.desktop");
        let mut f = File::create(&user_only).unwrap();
        writeln!(f, "[Desktop Entry]\nName=User Only").unwrap();

        let ignored_file = user_dir.join("ignored.txt");
        File::create(&ignored_file).unwrap();

        // Create files in sys_dir1:
        // app_common.desktop (shadowed by user), sys_common.desktop, sys_only1.desktop
        let sys1_common = sys_dir1.join("app_common.desktop");
        let mut f = File::create(&sys1_common).unwrap();
        writeln!(f, "[Desktop Entry]\nName=Sys1 Common").unwrap();

        let sys_common1 = sys_dir1.join("sys_common.desktop");
        let mut f = File::create(&sys_common1).unwrap();
        writeln!(f, "[Desktop Entry]\nName=Sys Common from Dir1").unwrap();

        let sys_only1 = sys_dir1.join("sys_only1.desktop");
        let mut f = File::create(&sys_only1).unwrap();
        writeln!(f, "[Desktop Entry]\nName=Sys Only 1").unwrap();

        // Create files in sys_dir2:
        // sys_common.desktop (shadowed by sys1), sys_only2.desktop
        let sys_common2 = sys_dir2.join("sys_common.desktop");
        let mut f = File::create(&sys_common2).unwrap();
        writeln!(f, "[Desktop Entry]\nName=Sys Common from Dir2").unwrap();

        let sys_only2 = sys_dir2.join("sys_only2.desktop");
        let mut f = File::create(&sys_only2).unwrap();
        writeln!(f, "[Desktop Entry]\nName=Sys Only 2").unwrap();

        let result = DesktopReader::list_desktop_files_from_dirs(
            Some(&user_dir),
            &[sys_dir1.clone(), sys_dir2.clone()],
        );

        // Expected files in result:
        // 1. user_dir/app_common.desktop
        // 2. user_dir/user_only.desktop
        // 3. sys_dir1/sys_common.desktop (from sys_dir1, not sys_dir2)
        // 4. sys_dir1/sys_only1.desktop
        // 5. sys_dir2/sys_only2.desktop
        assert_eq!(result.len(), 5);
        assert!(result.contains(&user_common));
        assert!(result.contains(&user_only));
        assert!(result.contains(&sys_common1));
        assert!(!result.contains(&sys_common2));
        assert!(result.contains(&sys_only1));
        assert!(result.contains(&sys_only2));
        assert!(!result.contains(&sys1_common)); // shadowed by user_common
        assert!(!result.contains(&ignored_file));

        let _ = fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn test_is_system_path_detection() {
        assert!(DesktopReader::is_system_path(Path::new(
            "/usr/share/applications/firefox.desktop"
        )));
        assert!(DesktopReader::is_system_path(Path::new(
            "/run/host/usr/share/applications/calc.desktop"
        )));
        assert!(DesktopReader::is_system_path(Path::new(
            "/var/lib/flatpak/exports/share/applications/app.desktop"
        )));
        assert!(DesktopReader::is_system_path(Path::new(
            "/opt/my_app/app.desktop"
        )));
    }

    #[test]
    fn test_detect_location() {
        assert_eq!(
            DesktopReader::detect_location(Path::new("/usr/share/applications/firefox.desktop")),
            DesktopLocation::System
        );
        assert_eq!(
            DesktopReader::detect_location(Path::new(
                "/run/host/usr/local/share/applications/editor.desktop"
            )),
            DesktopLocation::System
        );
        assert_eq!(
            DesktopReader::detect_location(Path::new(
                "/var/lib/flatpak/exports/share/applications/org.mozilla.firefox.desktop"
            )),
            DesktopLocation::Flatpak
        );
        assert_eq!(
            DesktopReader::detect_location(Path::new(
                "/run/host/var/lib/flatpak/exports/share/applications/org.gimp.GIMP.desktop"
            )),
            DesktopLocation::Flatpak
        );
        if let Some(user_dir) = DesktopReader::user_applications_dir() {
            assert_eq!(
                DesktopReader::detect_location(&user_dir.join("custom.desktop")),
                DesktopLocation::User
            );
        }
    }
}
