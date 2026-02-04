use gtk4::gio::SimpleAction;
use gtk4::{Application, ApplicationWindow, FileChooserAction, FileChooserDialog, ResponseType};
use adw::{ApplicationWindow as AdwApplicationWindow, prelude::*};

use crate::domain::desktop_entry::DesktopEntry;
use crate::services::desktop_reader::DesktopReader;
use crate::services::desktop_writer::DesktopWriter;
use crate::ui::dialogs;
use crate::ui::editor::entry_form::{EntryWidgets, collect_entry, set_form_from_entry};
use crate::ui::state::SharedState;

pub fn register_actions(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
    ensure_temp_row: impl Fn() + Clone + 'static,
    refresh_list: impl Fn() + Clone + 'static,
) {
    register_new_action(app, widgets, state.clone(), status_label, ensure_temp_row);
    register_open_action(app, widgets, state.clone(), status_label);
    register_save_action(app, widgets, state.clone(), status_label);
    register_refresh_action(app, refresh_list);
    register_quit_action(app);
    register_dir_actions(app, win);
    register_about_actions(app, win);
    register_fullscreen_action(win);
}

fn register_new_action(
    app: &Application,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
    ensure_temp_row: impl Fn() + Clone + 'static,
) {
    let action = SimpleAction::new("new", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    action.connect_activate(move |_, _| {
        set_form_from_entry(&w, &DesktopEntry::default());
        {
            let mut st = s.borrow_mut();
            st.selected_path = None;
            st.in_edit = true;
        }
        ensure_temp_row();
        w.type_combo.set_sensitive(true);
        lbl.set_text("New entry");
    });
    app.add_action(&action);
}

fn register_open_action(
    app: &Application,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
) {
    let action = SimpleAction::new("open", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    action.connect_activate(move |_, _| {
        let dialog = FileChooserDialog::new(
            Some("Open .desktop"),
            None::<&ApplicationWindow>,
            FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        let w2 = w.clone();
        let s2 = s.clone();
        let lbl2 = lbl.clone();
        dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        match DesktopReader::read_from_path(&path) {
                            Ok(de) => {
                                set_form_from_entry(&w2, &de);
                                w2.type_combo.set_sensitive(false);
                                s2.borrow_mut().selected_path = Some(path.clone());
                                lbl2.set_text(&path.to_string_lossy());
                            }
                            Err(e) => lbl2.set_text(&format!("Open failed: {}", e)),
                        }
                    }
                }
            }
            d.close();
        });
        dialog.show();
    });
    app.add_action(&action);
}

fn register_save_action(
    app: &Application,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
) {
    let action = SimpleAction::new("save", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    action.connect_activate(move |_, _| {
        match collect_entry(&w) {
            Ok(de) => {
                let sel_path = s.borrow().selected_path.clone();
                if let Some(path) = sel_path {
                    match DesktopWriter::write_to_path(&de, &path) {
                        Ok(_) => lbl.set_text(&format!("Updated: {}", path.display())),
                        Err(e) => lbl.set_text(&format!("Save failed: {}", e)),
                    }
                } else {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match DesktopWriter::write(&de, &fname, true) {
                        Ok(path) => lbl.set_text(&format!("Saved: {}", path.display())),
                        Err(e) => lbl.set_text(&format!("Save failed: {}", e)),
                    }
                }
            }
            Err(e) => lbl.set_text(&format!("Invalid: {}", e)),
        }
    });
    app.add_action(&action);
}

fn register_refresh_action(app: &Application, refresh_list: impl Fn() + 'static) {
    let action = SimpleAction::new("refresh", None);
    action.connect_activate(move |_, _| refresh_list());
    app.add_action(&action);
}

fn register_quit_action(app: &Application) {
    let action = SimpleAction::new("quit", None);
    let a = app.clone();
    action.connect_activate(move |_, _| a.quit());
    app.add_action(&action);
}

fn register_dir_actions(app: &Application, win: &AdwApplicationWindow) {
    let open_sys = SimpleAction::new("open_system_dir", None);
    let w = win.clone();
    open_sys.connect_activate(move |_, _| {
        #[cfg(target_os = "linux")]
        {
            let path = std::path::Path::new("/usr/share/applications");
            if let Err(e) = open::that(path) {
                dialogs::show_error(&w, &format!("Failed to open system dir: {}", e));
            }
        }
    });
    app.add_action(&open_sys);

    let open_user = SimpleAction::new("open_user_dir", None);
    let w = win.clone();
    open_user.connect_activate(move |_, _| {
        #[cfg(target_os = "linux")]
        {
            if let Some(base) = directories::BaseDirs::new() {
                let path = base.data_dir().join("applications");
                if let Err(e) = open::that(&path) {
                    dialogs::show_error(&w, &format!("Failed to open user dir: {}", e));
                }
            } else {
                dialogs::show_error(&w, "Cannot resolve user data dir");
            }
        }
    });
    app.add_action(&open_user);
}

fn register_about_actions(app: &Application, win: &AdwApplicationWindow) {
    let about = SimpleAction::new("about", None);
    let w = win.clone();
    about.connect_activate(move |_, _| dialogs::show_about(&w));
    app.add_action(&about);

    let credits = SimpleAction::new("credits", None);
    let w = win.clone();
    credits.connect_activate(move |_, _| dialogs::show_credits(&w));
    app.add_action(&credits);
}

fn register_fullscreen_action(win: &AdwApplicationWindow) {
    let action = SimpleAction::new("toggle_fullscreen", None);
    let w = win.clone();
    action.connect_activate(move |_, _| {
        if w.is_fullscreen() { w.unfullscreen(); } else { w.fullscreen(); }
    });
    win.add_action(&action);
}

pub fn do_save(
    widgets: &EntryWidgets,
    state: &SharedState,
    win: &impl IsA<gtk4::Window>,
) {
    match collect_entry(widgets) {
        Ok(de) => {
            let sel_path = state.borrow().selected_path.clone();
            if let Some(path) = sel_path {
                match DesktopWriter::write_to_path(&de, &path) {
                    Ok(_) => dialogs::show_save_success(win, path, true),
                    Err(e) => dialogs::show_error(win, &e.to_string()),
                }
            } else {
                let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                match DesktopWriter::write(&de, &fname, true) {
                    Ok(path) => dialogs::show_save_success(win, path, false),
                    Err(e) => dialogs::show_error(win, &e.to_string()),
                }
            }
        }
        Err(e) => dialogs::show_error(win, &e),
    }
}

pub fn do_preview(widgets: &EntryWidgets, win: &impl IsA<gtk4::Window>) {
    match collect_entry(widgets) {
        Ok(de) => dialogs::show_preview(win, &de.to_ini_string()),
        Err(e) => dialogs::show_error(win, &e),
    }
}

pub fn do_delete(
    state: &SharedState,
    widgets: &EntryWidgets,
    win: &impl IsA<gtk4::Window>,
    status_label: &gtk4::Label,
    refresh_list: impl Fn() + 'static,
) {
    let maybe_path = state.borrow().selected_path.clone();
    if let Some(path) = maybe_path {
        let s = state.clone();
        let w = widgets.clone();
        let lbl = status_label.clone();
        let win_clone = win.clone();
        let path_clone = path.clone();
        dialogs::confirm_delete(win, &path, move || {
            if let Err(e) = std::fs::remove_file(&path_clone) {
                dialogs::show_error(&win_clone, &format!("Failed to delete: {}", e));
            } else {
                set_form_from_entry(&w, &DesktopEntry::default());
                w.type_combo.set_sensitive(true);
                s.borrow_mut().selected_path = None;
                refresh_list();
                lbl.set_text("Deleted");
            }
        });
    } else {
        dialogs::show_error(win, "No file selected to delete");
    }
}
