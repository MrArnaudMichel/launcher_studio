use gtk4 as gtk;

pub struct IconService {
    icon_theme: gtk::IconTheme,
}

impl IconService {
    pub fn new() -> Self {
        let display = gtk::gdk::Display::default().expect("No display");
        let icon_theme = gtk::IconTheme::for_display(&display);
        Self { icon_theme }
    }

    pub fn list_icons(&self) -> Vec<String> {
        let mut names: Vec<String> = self.icon_theme.icon_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        names.sort();
        names
    }

    pub fn search_icons(&self, query: &str) -> Vec<String> {
        let query = query.to_lowercase();
        self.list_icons().into_iter()
            .filter(|name| name.to_lowercase().contains(&query))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_icons_not_empty() {
        // Skip GTK init if it fails in CI/headless, but try to test logic if possible.
        if gtk::init().is_ok() {
            let service = IconService::new();
            let icons = service.list_icons();
            assert!(!icons.is_empty(), "Icon theme should have some icons");
        }
    }

    #[test]
    fn test_search_icons() {
        if gtk::init().is_ok() {
            let service = IconService::new();
            let results = service.search_icons("folder");
            for res in &results {
                assert!(res.to_lowercase().contains("folder"));
            }
        }
    }
}
