use gtk4::{Box as GtkBox, Image, Label, ListBox, ListBoxRow, Orientation};
use gtk4::prelude::*;
use std::path::PathBuf;
use crate::services::desktop_reader::DesktopReader;
use crate::ui::state::SharedState;
use crate::ui::editor::entry_form::{EntryWidgets, set_form_from_entry};
pub fn refresh_desktop_list(
    listbox: &ListBox,
    state: &SharedState,
    status_label: &Label,
    ensure_temp_row: &dyn Fn(),
) {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }
    match DesktopReader::list_desktop_files() {
        Ok(paths) => {
            for path in paths {
                let (name, icon_str) = match DesktopReader::read_from_path(&path) {
                    Ok(de) => (de.name, de.icon),
                    Err(_) => (
                        path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                        None,
                    ),
                };
                let row = create_list_row(&name, icon_str.as_deref(), &path);
                listbox.append(&row);
            }
            status_label.set_text("List refreshed");
            if state.borrow().in_edit {
                ensure_temp_row();
            }
        }
        Err(e) => status_label.set_text(&format!("Failed to list: {}", e)),
    }
}
pub fn create_list_row(name: &str, icon: Option<&str>, path: &PathBuf) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hb = GtkBox::new(Orientation::Horizontal, 6);
    let img = match icon {
        Some(i) if i.contains('/') => Image::from_file(i),
        Some(i) => Image::from_icon_name(i),
        None => Image::from_icon_name("application-x-executable-symbolic"),
    };
    img.set_pixel_size(16);
    hb.append(&img);
    let lbl = Label::new(Some(name));
    lbl.set_xalign(0.0);
    hb.append(&lbl);
    row.set_child(Some(&hb));
    row.set_selectable(true);
    row.add_css_class("activatable");
    row.set_widget_name(&path.to_string_lossy());
    row
}
pub fn create_temp_row(listbox: &ListBox, state: &SharedState, name_entry: &gtk4::Entry, icon_entry: &gtk4::Entry) {
    if let Some(old_row) = state.borrow_mut().temp_row.take() {
        listbox.remove(&old_row);
    }
    let name = {
        let n = name_entry.text().to_string();
        if n.trim().is_empty() { "(New entry)".to_string() } else { n }
    };
    let icon_txt = icon_entry.text().to_string();
    let img = if icon_txt.trim().is_empty() {
        Image::from_icon_name("application-x-executable-symbolic")
    } else if icon_txt.contains('/') {
        Image::from_file(icon_txt)
    } else {
        Image::from_icon_name(&icon_txt)
    };
    img.set_pixel_size(16);
    let row = ListBoxRow::new();
    let hb = GtkBox::new(Orientation::Horizontal, 6);
    hb.append(&img);
    let lbl = Label::new(Some(&name));
    lbl.set_xalign(0.0);
    hb.append(&lbl);
    row.set_child(Some(&hb));
    row.set_selectable(true);
    row.set_sensitive(false);
    row.add_css_class("activatable");
    row.set_widget_name(":unsaved");
    listbox.append(&row);
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
            state.borrow_mut().selected_path = Some(path.clone());
            status_label.set_text(&path.to_string_lossy());
        }
        Err(e) => {
            status_label.set_text(&format!("Open failed: {}", e));
        }
    }
}
