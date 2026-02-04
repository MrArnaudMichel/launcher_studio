use gtk4::{Image, ToggleButton};
use gtk4::prelude::*;
use adw::{StyleManager, ColorScheme};
use std::cell::RefCell;
use std::rc::Rc;

const NIGHT_NAME: &str = "launcher-studio-weather-night-symbolic";
const SUNNY_NAME: &str = "launcher-studio-weather-sunny-symbolic";
const NIGHT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/launcher-studio-weather-night-symbolic.svg");
const SUNNY_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/launcher-studio-weather-sunny-symbolic.svg");
const NIGHT_EMBED: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/launcher-studio-weather-night-symbolic.svg"));
const SUNNY_EMBED: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/launcher-studio-weather-sunny-symbolic.svg"));

fn write_embedded_icon(name: &str, bytes: &[u8]) -> Option<std::path::PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = dir.join(name);
    if path.exists() {
        return Some(path);
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, bytes).ok().map(|_| path)
}

fn set_theme_icon(icon: &Image, is_dark: bool, file_mode: bool) {
    if file_mode {
        let path = if is_dark { SUNNY_FILE } else { NIGHT_FILE };
        if std::path::Path::new(path).exists() {
            icon.set_from_file(Some(path));
        } else {
            let embed = if is_dark {
                write_embedded_icon("launcher-studio-weather-sunny-symbolic.svg", SUNNY_EMBED)
            } else {
                write_embedded_icon("launcher-studio-weather-night-symbolic.svg", NIGHT_EMBED)
            };
            if let Some(p) = embed.and_then(|p| p.to_str().map(String::from)) {
                icon.set_from_file(Some(&p));
            } else {
                icon.set_icon_name(Some(if is_dark { SUNNY_NAME } else { NIGHT_NAME }));
            }
        }
    } else {
        icon.set_icon_name(Some(if is_dark { SUNNY_NAME } else { NIGHT_NAME }));
    }
}

pub fn create_theme_button() -> ToggleButton {
    let btn = ToggleButton::new();
    let icon = Image::new();
    icon.set_pixel_size(16);
    btn.set_child(Some(&icon));
    btn.set_tooltip_text(Some("Toggle dark theme"));

    let style_manager = StyleManager::default();
    let is_dark = style_manager.is_dark();
    btn.set_active(is_dark);

    let mut file_mode = true;
    if let Some(display) = gtk4::gdk::Display::default() {
        let icon_theme = gtk4::IconTheme::for_display(&display);
        if icon_theme.has_icon(NIGHT_NAME) && icon_theme.has_icon(SUNNY_NAME) {
            file_mode = false;
        }
    }

    set_theme_icon(&icon, is_dark, file_mode);

    let file_mode_rc = Rc::new(RefCell::new(file_mode));
    let icon_c = icon.clone();
    let file_mode_c = file_mode_rc.clone();

    btn.connect_toggled(move |b| {
        let active = b.is_active();
        let sm = StyleManager::default();
        sm.set_color_scheme(if active { ColorScheme::ForceDark } else { ColorScheme::ForceLight });
        set_theme_icon(&icon_c, active, *file_mode_c.borrow());
    });

    btn
}
