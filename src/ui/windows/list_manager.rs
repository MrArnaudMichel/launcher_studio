use crate::services::desktop_reader::{DesktopLocation, DesktopReader};
use crate::ui::components::sidebar::Sidebar;
use crate::ui::editor::entry_form::{EntryWidgets, set_form_from_entry};
use crate::ui::state::SharedState;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Image, Label, ListBox, ListBoxRow, Orientation};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LauncherItemInfo {
    pub name: String,
    pub generic_name: String,
    pub comment: String,
    pub exec: String,
    pub categories: String,
    pub filename: String,
    pub path: PathBuf,
    pub location: DesktopLocation,
}

impl LauncherItemInfo {
    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.name.to_lowercase().contains(&q)
            || self.filename.to_lowercase().contains(&q)
            || self.generic_name.to_lowercase().contains(&q)
            || self.comment.to_lowercase().contains(&q)
            || self.exec.to_lowercase().contains(&q)
            || self.categories.to_lowercase().contains(&q)
            || self.path.to_string_lossy().to_lowercase().contains(&q)
    }
}

pub type LauncherItemsMap = Rc<RefCell<HashMap<String, LauncherItemInfo>>>;
pub type FilterState = Rc<RefCell<DesktopLocation>>;
pub type SearchState = Rc<RefCell<String>>;

pub struct LauncherListManager {
    pub items_map: LauncherItemsMap,
    pub filter_state: FilterState,
    pub search_state: SearchState,
    pub sidebar: Sidebar,
}

impl LauncherListManager {
    pub fn new(sidebar: Sidebar) -> Rc<Self> {
        let items_map = Rc::new(RefCell::new(HashMap::new()));
        let filter_state = Rc::new(RefCell::new(DesktopLocation::All));
        let search_state = Rc::new(RefCell::new(String::new()));

        let mgr = Rc::new(Self {
            items_map,
            filter_state,
            search_state,
            sidebar,
        });

        mgr.setup_filtering();
        mgr
    }

    fn setup_filtering(self: &Rc<Self>) {
        // Setup filter closure for listbox
        let items_map = self.items_map.clone();
        let filter_state = self.filter_state.clone();
        let search_state = self.search_state.clone();

        self.sidebar.listbox.set_filter_func(move |row| {
            let widget_name = row.widget_name();
            if widget_name == ":unsaved" {
                return true;
            }
            let map = items_map.borrow();
            if let Some(info) = map.get(widget_name.as_str()) {
                let current_loc = *filter_state.borrow();
                let loc_match = match current_loc {
                    DesktopLocation::All => true,
                    DesktopLocation::User => info.location == DesktopLocation::User,
                    DesktopLocation::System => info.location == DesktopLocation::System,
                    DesktopLocation::Flatpak => info.location == DesktopLocation::Flatpak,
                };
                if !loc_match {
                    return false;
                }
                let query = search_state.borrow();
                info.matches_query(&query)
            } else {
                true
            }
        });

        // Setup search entry change
        {
            let mgr = self.clone();
            self.sidebar
                .search_entry
                .connect_search_changed(move |entry| {
                    *mgr.search_state.borrow_mut() = entry.text().to_string();
                    mgr.sidebar.listbox.invalidate_filter();
                    mgr.update_ui_state();
                });
        }

        // Setup filter button clicks
        {
            let mgr = self.clone();
            self.sidebar.btn_all.connect_toggled(move |btn| {
                if btn.is_active() {
                    *mgr.filter_state.borrow_mut() = DesktopLocation::All;
                    mgr.sidebar.listbox.invalidate_filter();
                    mgr.update_ui_state();
                }
            });
        }
        {
            let mgr = self.clone();
            self.sidebar.btn_home.connect_toggled(move |btn| {
                if btn.is_active() {
                    *mgr.filter_state.borrow_mut() = DesktopLocation::User;
                    mgr.sidebar.listbox.invalidate_filter();
                    mgr.update_ui_state();
                }
            });
        }
        {
            let mgr = self.clone();
            self.sidebar.btn_system.connect_toggled(move |btn| {
                if btn.is_active() {
                    *mgr.filter_state.borrow_mut() = DesktopLocation::System;
                    mgr.sidebar.listbox.invalidate_filter();
                    mgr.update_ui_state();
                }
            });
        }
        {
            let mgr = self.clone();
            self.sidebar.btn_flatpak.connect_toggled(move |btn| {
                if btn.is_active() {
                    *mgr.filter_state.borrow_mut() = DesktopLocation::Flatpak;
                    mgr.sidebar.listbox.invalidate_filter();
                    mgr.update_ui_state();
                }
            });
        }

        // Setup clear button in empty state
        {
            let mgr = self.clone();
            self.sidebar.clear_btn.connect_clicked(move |_| {
                mgr.sidebar.search_entry.set_text("");
                mgr.sidebar.btn_all.set_active(true);
                *mgr.search_state.borrow_mut() = String::new();
                *mgr.filter_state.borrow_mut() = DesktopLocation::All;
                mgr.sidebar.listbox.invalidate_filter();
                mgr.update_ui_state();
            });
        }
    }

    pub fn update_ui_state(&self) {
        let map = self.items_map.borrow();
        let total = map.len();
        let user_count = map
            .values()
            .filter(|i| i.location == DesktopLocation::User)
            .count();
        let sys_count = map
            .values()
            .filter(|i| i.location == DesktopLocation::System)
            .count();
        let flatpak_count = map
            .values()
            .filter(|i| i.location == DesktopLocation::Flatpak)
            .count();

        let cur_filter = *self.filter_state.borrow();
        let cur_query = self.search_state.borrow();

        let visible_count = map
            .values()
            .filter(|i| {
                let loc_match = match cur_filter {
                    DesktopLocation::All => true,
                    DesktopLocation::User => i.location == DesktopLocation::User,
                    DesktopLocation::System => i.location == DesktopLocation::System,
                    DesktopLocation::Flatpak => i.location == DesktopLocation::Flatpak,
                };
                loc_match && i.matches_query(&cur_query)
            })
            .count();

        self.sidebar.btn_all.set_label(&format!("All ({})", total));
        self.sidebar
            .btn_home
            .set_label(&format!("Home ({})", user_count));
        self.sidebar
            .btn_system
            .set_label(&format!("/usr ({})", sys_count));
        self.sidebar
            .btn_flatpak
            .set_label(&format!("Flatpak ({})", flatpak_count));

        if cur_query.trim().is_empty() {
            match cur_filter {
                DesktopLocation::All => self
                    .sidebar
                    .count_label
                    .set_text(&format!("{} applications available", total)),
                DesktopLocation::User => self
                    .sidebar
                    .count_label
                    .set_text(&format!("{} applications in Home (~)", user_count)),
                DesktopLocation::System => self
                    .sidebar
                    .count_label
                    .set_text(&format!("{} applications in /usr/share", sys_count)),
                DesktopLocation::Flatpak => self
                    .sidebar
                    .count_label
                    .set_text(&format!("{} Flatpak applications", flatpak_count)),
            }
        } else {
            self.sidebar.count_label.set_text(&format!(
                "Showing {} of {} applications",
                visible_count, total
            ));
        }

        if visible_count == 0 && total > 0 {
            self.sidebar.stack.set_visible_child_name("empty");
        } else {
            self.sidebar.stack.set_visible_child_name("list");
        }
    }

    pub fn refresh_list(
        &self,
        state: &SharedState,
        status_label: &Label,
        ensure_temp_row: &dyn Fn(),
    ) {
        while let Some(child) = self.sidebar.listbox.first_child() {
            self.sidebar.listbox.remove(&child);
        }
        self.items_map.borrow_mut().clear();

        match DesktopReader::list_desktop_files() {
            Ok(paths) => {
                for path in paths {
                    let location = DesktopReader::detect_location(&path);
                    let filename = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();

                    let (name, icon_str, generic_name, comment, exec, categories) =
                        match DesktopReader::read_from_path(&path) {
                            Ok(de) => {
                                let n = if !de.name.trim().is_empty() {
                                    de.name
                                } else {
                                    path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Unnamed")
                                        .to_string()
                                };
                                (
                                    n,
                                    de.icon,
                                    de.generic_name.unwrap_or_default(),
                                    de.comment.unwrap_or_default(),
                                    de.exec,
                                    de.categories.join(" "),
                                )
                            }
                            Err(_) => (
                                path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("")
                                    .to_string(),
                                None,
                                String::new(),
                                String::new(),
                                String::new(),
                                String::new(),
                            ),
                        };

                    let info = LauncherItemInfo {
                        name: name.clone(),
                        generic_name,
                        comment,
                        exec,
                        categories,
                        filename: filename.clone(),
                        path: path.clone(),
                        location,
                    };

                    self.items_map
                        .borrow_mut()
                        .insert(path.to_string_lossy().to_string(), info);

                    let row =
                        create_list_row(&name, icon_str.as_deref(), &path, location, &filename);
                    self.sidebar.listbox.append(&row);
                }

                self.sidebar.listbox.invalidate_filter();
                self.update_ui_state();
                status_label.set_text("List refreshed");
                if state.borrow().in_edit {
                    ensure_temp_row();
                }
            }
            Err(e) => status_label.set_text(&format!("Failed to list: {}", e)),
        }
    }
}

pub fn create_list_row(
    name: &str,
    icon: Option<&str>,
    path: &Path,
    location: DesktopLocation,
    filename: &str,
) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hb = GtkBox::new(Orientation::Horizontal, 10);
    hb.set_margin_start(8);
    hb.set_margin_end(8);
    hb.set_margin_top(6);
    hb.set_margin_bottom(6);

    let img = match icon {
        Some(i) if i.contains('/') => Image::from_file(i),
        Some(i) => Image::from_icon_name(i),
        None => Image::from_icon_name("application-x-executable-symbolic"),
    };
    img.set_pixel_size(24);
    img.set_valign(Align::Center);
    hb.append(&img);

    let vbox = GtkBox::new(Orientation::Vertical, 2);
    vbox.set_hexpand(true);
    vbox.set_valign(Align::Center);

    let lbl = Label::new(Some(name));
    lbl.set_xalign(0.0);
    lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    lbl.add_css_class("heading");
    vbox.append(&lbl);

    let sub_box = GtkBox::new(Orientation::Horizontal, 6);
    sub_box.set_hexpand(true);

    let fn_lbl = Label::new(Some(filename));
    fn_lbl.add_css_class("dim-label");
    fn_lbl.add_css_class("caption");
    fn_lbl.set_xalign(0.0);
    fn_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    fn_lbl.set_hexpand(true);
    sub_box.append(&fn_lbl);

    let badge = Label::new(Some(location.badge_label()));
    badge.add_css_class("caption");
    badge.add_css_class("dim-label");
    badge.set_xalign(1.0);
    sub_box.append(&badge);

    vbox.append(&sub_box);
    hb.append(&vbox);

    row.set_child(Some(&hb));
    row.set_selectable(true);
    row.add_css_class("activatable");
    row.set_widget_name(&path.to_string_lossy());

    let tooltip = format!(
        "Name: {}\nFile: {}\nLocation: {} ({})\n{}",
        name,
        filename,
        location.label(),
        path.display(),
        if location == DesktopLocation::System {
            "System launcher (Read-only: edits are saved as user override in ~/.local/share/applications)"
        } else {
            "User launcher"
        }
    );
    row.set_tooltip_text(Some(&tooltip));
    row
}

pub fn create_temp_row(
    listbox: &ListBox,
    state: &SharedState,
    name_entry: &gtk4::Entry,
    icon_entry: &gtk4::Entry,
) {
    if let Some(old_row) = state.borrow_mut().temp_row.take() {
        listbox.remove(&old_row);
    }
    let name = {
        let n = name_entry.text().to_string();
        if n.trim().is_empty() {
            "(New entry)".to_string()
        } else {
            n
        }
    };
    let icon_txt = icon_entry.text().to_string();
    let img = if icon_txt.trim().is_empty() {
        Image::from_icon_name("application-x-executable-symbolic")
    } else if icon_txt.contains('/') {
        Image::from_file(icon_txt)
    } else {
        Image::from_icon_name(&icon_txt)
    };
    img.set_pixel_size(24);
    img.set_valign(Align::Center);

    let row = ListBoxRow::new();
    let hb = GtkBox::new(Orientation::Horizontal, 10);
    hb.set_margin_start(8);
    hb.set_margin_end(8);
    hb.set_margin_top(6);
    hb.set_margin_bottom(6);
    hb.append(&img);

    let vbox = GtkBox::new(Orientation::Vertical, 2);
    vbox.set_hexpand(true);
    vbox.set_valign(Align::Center);

    let lbl = Label::new(Some(&name));
    lbl.set_xalign(0.0);
    lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    lbl.add_css_class("heading");
    vbox.append(&lbl);

    let sub_box = GtkBox::new(Orientation::Horizontal, 6);
    let draft_lbl = Label::new(Some("Draft (unsaved)"));
    draft_lbl.add_css_class("dim-label");
    draft_lbl.add_css_class("caption");
    draft_lbl.set_xalign(0.0);
    sub_box.append(&draft_lbl);

    vbox.append(&sub_box);
    hb.append(&vbox);

    row.set_child(Some(&hb));
    row.set_selectable(true);
    row.set_sensitive(false);
    row.add_css_class("activatable");
    row.set_widget_name(":unsaved");
    listbox.prepend(&row);
    listbox.select_row(Some(&row));
    state.borrow_mut().temp_row = Some(row);
}

pub fn remove_temp_row(listbox: &ListBox, state: &SharedState) {
    if let Some(row) = state.borrow_mut().temp_row.take() {
        listbox.remove(&row);
    }
}

pub fn on_row_activated(
    row: &ListBoxRow,
    state: &SharedState,
    widgets: &EntryWidgets,
    status_label: &Label,
    listbox: &ListBox,
) {
    if row.widget_name() == ":unsaved" {
        return;
    }
    state.borrow_mut().in_edit = false;
    remove_temp_row(listbox, state);
    let path = PathBuf::from(row.widget_name().to_string());
    match DesktopReader::read_from_path(&path) {
        Ok(de) => {
            set_form_from_entry(widgets, &de);
            widgets.type_combo.set_sensitive(false);
            let mut st = state.borrow_mut();
            st.selected_path = Some(path.clone());
            st.is_dirty = false;
            status_label.set_text(&path.to_string_lossy());
        }
        Err(e) => {
            status_label.set_text(&format!("Open failed: {}", e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_item_info_matches_query() {
        let item = LauncherItemInfo {
            name: "Visual Studio Code".to_string(),
            generic_name: "Code Editor".to_string(),
            comment: "Edit, debug and build modern web and cloud apps".to_string(),
            exec: "/usr/bin/code --no-sandbox %F".to_string(),
            categories: "Development;IDE;TextEditor;".to_string(),
            filename: "code.desktop".to_string(),
            path: PathBuf::from("/usr/share/applications/code.desktop"),
            location: DesktopLocation::System,
        };

        // Empty query matches everything
        assert!(item.matches_query(""));
        assert!(item.matches_query("   "));

        // Match by Name
        assert!(item.matches_query("visual"));
        assert!(item.matches_query("CODE"));

        // Match by Generic Name
        assert!(item.matches_query("editor"));

        // Match by Comment
        assert!(item.matches_query("debug"));

        // Match by Exec
        assert!(item.matches_query("no-sandbox"));

        // Match by Categories
        assert!(item.matches_query("development"));
        assert!(item.matches_query("texteditor"));

        // Match by Filename
        assert!(item.matches_query("code.desktop"));

        // Match by Path
        assert!(item.matches_query("/usr/share"));

        // Non-matching query
        assert!(!item.matches_query("photoshop"));
        assert!(!item.matches_query("calculator"));
    }
}
