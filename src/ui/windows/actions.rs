use adw::{ApplicationWindow as AdwApplicationWindow, prelude::*};
use gtk4::gio::SimpleAction;
use gtk4::{Application, FileChooserAction, FileChooserDialog, FileFilter, ResponseType};
use std::path::{Path, PathBuf};

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
    register_new_action(
        app,
        win,
        widgets,
        state.clone(),
        status_label,
        ensure_temp_row,
    );
    register_open_action(app, win, widgets, state.clone(), status_label);
    register_import_system_action(app, win, widgets, state.clone(), status_label);
    register_save_action(
        app,
        win,
        widgets,
        state.clone(),
        status_label,
        refresh_list.clone(),
    );
    register_save_as_action(
        app,
        win,
        widgets,
        state.clone(),
        status_label,
        refresh_list.clone(),
    );
    register_refresh_action(app, refresh_list);
    register_quit_action(app, win, state.clone());
    register_dir_actions(app, win);
    register_about_actions(app, win);
    register_fullscreen_action(win);
    register_shortcuts(app);
}

fn register_shortcuts(app: &Application) {
    app.set_accels_for_action("app.new", &["<Ctrl>N"]);
    app.set_accels_for_action("app.open", &["<Ctrl>O"]);
    app.set_accels_for_action("app.import_system", &["<Ctrl><Alt>O", "<Ctrl>I"]);
    app.set_accels_for_action("app.save", &["<Ctrl>S"]);
    app.set_accels_for_action("app.save_as", &["<Ctrl><Shift>S"]);
    app.set_accels_for_action("app.refresh", &["F5"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);
    app.set_accels_for_action("win.toggle_fullscreen", &["F11"]);
}

fn register_new_action(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
    ensure_temp_row: impl Fn() + Clone + 'static,
) {
    let action = SimpleAction::new("new", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    let wwin = win.clone();
    action.connect_activate(move |_, _| {
        let w2 = w.clone();
        let s2 = s.clone();
        let lbl2 = lbl.clone();
        let ensure2 = ensure_temp_row.clone();
        run_after_unsaved_confirmation(&wwin, &s, move || {
            do_new(&w2, &s2, &lbl2, ensure2.clone());
        });
    });
    app.add_action(&action);
}

fn register_open_action(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
) {
    let action = SimpleAction::new("open", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    let wwin = win.clone();
    action.connect_activate(move |_, _| {
        let w2 = w.clone();
        let s2 = s.clone();
        let lbl2 = lbl.clone();
        let open_win = wwin.clone();
        run_after_unsaved_confirmation(&wwin, &s, move || {
            do_open(&w2, &s2, &lbl2, &open_win);
        });
    });
    app.add_action(&action);
}

fn register_import_system_action(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
) {
    let action = SimpleAction::new("import_system", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    let wwin = win.clone();
    action.connect_activate(move |_, _| {
        let w2 = w.clone();
        let s2 = s.clone();
        let lbl2 = lbl.clone();
        let open_win = wwin.clone();
        run_after_unsaved_confirmation(&wwin, &s, move || {
            do_import_system(&w2, &s2, &lbl2, &open_win);
        });
    });
    app.add_action(&action);
}

fn register_save_action(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
    refresh_list: impl Fn() + Clone + 'static,
) {
    let action = SimpleAction::new("save", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    let wwin = win.clone();
    let rl = refresh_list.clone();
    action.connect_activate(move |_, _| {
        do_save(&w, &s, &wwin, &lbl, {
            let r = rl.clone();
            move || r()
        });
    });
    app.add_action(&action);
}

fn register_save_as_action(
    app: &Application,
    win: &AdwApplicationWindow,
    widgets: &EntryWidgets,
    state: SharedState,
    status_label: &gtk4::Label,
    refresh_list: impl Fn() + Clone + 'static,
) {
    let action = SimpleAction::new("save_as", None);
    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    let wwin = win.clone();
    let rl = refresh_list.clone();
    action.connect_activate(move |_, _| {
        let lbl2 = lbl.clone();
        let wwin2 = wwin.clone();
        let rl2 = rl.clone();
        do_save_as(&w, &s, &wwin, move |path| {
            lbl2.set_text(&path.to_string_lossy());
            rl2();
            dialogs::show_save_success(&wwin2, path, false);
        });
    });
    app.add_action(&action);
}

fn register_refresh_action(app: &Application, refresh_list: impl Fn() + 'static) {
    let action = SimpleAction::new("refresh", None);
    action.connect_activate(move |_, _| refresh_list());
    app.add_action(&action);
}

fn register_quit_action(app: &Application, win: &AdwApplicationWindow, state: SharedState) {
    let action = SimpleAction::new("quit", None);
    let a = app.clone();
    let w = win.clone();
    let s = state.clone();
    action.connect_activate(move |_, _| {
        run_after_unsaved_confirmation(&w, &s, {
            let a2 = a.clone();
            move || a2.quit()
        });
    });
    app.add_action(&action);
}

fn register_dir_actions(app: &Application, win: &AdwApplicationWindow) {
    let open_sys = SimpleAction::new("open_system_dir", None);
    let w = win.clone();
    open_sys.connect_activate(move |_, _| {
        #[cfg(target_os = "linux")]
        {
            let path = DesktopReader::primary_system_applications_dir()
                .unwrap_or_else(|| PathBuf::from("/usr/share/applications"));
            open_folder(&w, &path);
        }
    });
    app.add_action(&open_sys);

    let open_user = SimpleAction::new("open_user_dir", None);
    let w = win.clone();
    open_user.connect_activate(move |_, _| {
        #[cfg(target_os = "linux")]
        {
            if let Some(path) = DesktopReader::user_applications_dir() {
                open_folder(&w, &path);
            } else {
                dialogs::show_error(&w, "Cannot resolve user applications directory");
            }
        }
    });
    app.add_action(&open_user);
}

fn open_folder(win: &AdwApplicationWindow, path: &Path) {
    #[cfg(target_os = "linux")]
    {
        let uri = format!("file://{}", path.display());
        let app_ctx = gtk4::gio::AppLaunchContext::new();
        if gtk4::gio::AppInfo::launch_default_for_uri(&uri, Some(&app_ctx)).is_err()
            && let Err(e) = open::that(path)
        {
            dialogs::show_error(win, &format!("Failed to open {}: {}", path.display(), e));
        }
    }
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
        if w.is_fullscreen() {
            w.unfullscreen();
        } else {
            w.fullscreen();
        }
    });
    win.add_action(&action);
}

fn run_after_unsaved_confirmation(
    win: &impl IsA<gtk4::Window>,
    state: &SharedState,
    action: impl Fn() + 'static,
) {
    if state.borrow().is_dirty {
        dialogs::confirm_discard_changes(win, action);
    } else {
        action();
    }
}

pub fn do_new(
    widgets: &EntryWidgets,
    state: &SharedState,
    status_label: &gtk4::Label,
    ensure_temp_row: impl Fn(),
) {
    set_form_from_entry(widgets, &DesktopEntry::default());
    {
        let mut st = state.borrow_mut();
        st.selected_path = None;
        st.in_edit = true;
        st.is_dirty = false;
    }
    ensure_temp_row();
    widgets.type_combo.set_sensitive(true);
    status_label.set_text("New entry");
}

pub fn do_open(
    widgets: &EntryWidgets,
    state: &SharedState,
    status_label: &gtk4::Label,
    win: &impl IsA<gtk4::Window>,
) {
    let dialog = FileChooserDialog::new(
        Some("Open .desktop"),
        Some(win.upcast_ref::<gtk4::Window>()),
        FileChooserAction::Open,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Open", ResponseType::Accept),
        ],
    );

    if let Some(path) = DesktopReader::user_applications_dir()
        && path.exists()
    {
        let _ = dialog.add_shortcut_folder(&gtk4::gio::File::for_path(&path));
        let _ = dialog.set_current_folder(Some(&gtk4::gio::File::for_path(&path)));
    }

    for sys_dir in DesktopReader::system_applications_dirs() {
        if sys_dir.exists() {
            let _ = dialog.add_shortcut_folder(&gtk4::gio::File::for_path(&sys_dir));
        }
    }

    let filter = FileFilter::new();
    filter.set_name(Some("Desktop files (*.desktop)"));
    filter.add_pattern("*.desktop");
    dialog.add_filter(&filter);

    let all_filter = FileFilter::new();
    all_filter.set_name(Some("All files"));
    all_filter.add_pattern("*");
    dialog.add_filter(&all_filter);

    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept
            && let Some(file) = d.file()
            && let Some(path) = file.path()
        {
            match DesktopReader::read_from_path(&path) {
                Ok(de) => {
                    set_form_from_entry(&w, &de);
                    w.type_combo.set_sensitive(false);
                    let mut st = s.borrow_mut();
                    st.selected_path = Some(path.clone());
                    st.in_edit = false;
                    st.is_dirty = false;
                    lbl.set_text(&path.to_string_lossy());
                }
                Err(e) => lbl.set_text(&format!("Open failed: {}", e)),
            }
        }
        d.close();
    });
    dialog.show();
}

pub fn do_import_system(
    widgets: &EntryWidgets,
    state: &SharedState,
    status_label: &gtk4::Label,
    win: &impl IsA<gtk4::Window>,
) {
    let dialog = FileChooserDialog::new(
        Some("Import System Launcher (/usr/share/applications)"),
        Some(win.upcast_ref::<gtk4::Window>()),
        FileChooserAction::Open,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Import", ResponseType::Accept),
        ],
    );

    if let Some(sys_path) = DesktopReader::primary_system_applications_dir() {
        let _ = dialog.set_current_folder(Some(&gtk4::gio::File::for_path(&sys_path)));
    }
    for sys_dir in DesktopReader::system_applications_dirs() {
        if sys_dir.exists() {
            let _ = dialog.add_shortcut_folder(&gtk4::gio::File::for_path(&sys_dir));
        }
    }

    let filter = FileFilter::new();
    filter.set_name(Some("Desktop files (*.desktop)"));
    filter.add_pattern("*.desktop");
    dialog.add_filter(&filter);

    let w = widgets.clone();
    let s = state.clone();
    let lbl = status_label.clone();
    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept
            && let Some(file) = d.file()
            && let Some(path) = file.path()
        {
            match DesktopReader::read_from_path(&path) {
                Ok(de) => {
                    set_form_from_entry(&w, &de);
                    w.type_combo.set_sensitive(false);
                    let mut st = s.borrow_mut();
                    st.selected_path = Some(path.clone());
                    st.in_edit = false;
                    st.is_dirty = false;
                    lbl.set_text(&format!("Imported: {}", path.to_string_lossy()));
                }
                Err(e) => lbl.set_text(&format!("Import failed: {}", e)),
            }
        }
        d.close();
    });
    dialog.show();
}

fn save_entry(widgets: &EntryWidgets, state: &SharedState) -> Result<(PathBuf, bool), String> {
    let de = collect_entry(widgets)?;
    let sel_path = state.borrow().selected_path.clone();
    let result = if let Some(path) = sel_path {
        if DesktopReader::is_system_path(&path) {
            // Save as user override in ~/.local/share/applications
            let fname = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("desktop-entry");
            DesktopWriter::write(&de, fname, true)
                .map(|p| (p, false))
                .map_err(|e| e.to_string())
        } else {
            DesktopWriter::write_to_path(&de, &path)
                .map(|p| (p, true))
                .map_err(|e| e.to_string())
        }
    } else {
        let fname = if !de.name.trim().is_empty() {
            de.name.clone()
        } else {
            "desktop-entry".into()
        };
        DesktopWriter::write(&de, &fname, true)
            .map(|p| (p, false))
            .map_err(|e| e.to_string())
    }?;

    let saved_path = result.0.clone();
    let mut st = state.borrow_mut();
    st.selected_path = Some(saved_path);
    st.is_dirty = false;
    Ok(result)
}

pub fn do_save_as(
    widgets: &EntryWidgets,
    state: &SharedState,
    win: &impl IsA<gtk4::Window>,
    on_success: impl Fn(PathBuf) + 'static,
) {
    let de = match collect_entry(widgets) {
        Ok(de) => de,
        Err(e) => {
            dialogs::show_error(win, &e);
            return;
        }
    };

    let dialog = FileChooserDialog::new(
        Some("Save .desktop as"),
        Some(win.upcast_ref::<gtk4::Window>()),
        FileChooserAction::Save,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Save", ResponseType::Accept),
        ],
    );

    if let Some(path) = DesktopReader::user_applications_dir()
        && path.exists()
    {
        let _ = dialog.add_shortcut_folder(&gtk4::gio::File::for_path(&path));
        let _ = dialog.set_current_folder(Some(&gtk4::gio::File::for_path(&path)));
    }

    for sys_dir in DesktopReader::system_applications_dirs() {
        if sys_dir.exists() {
            let _ = dialog.add_shortcut_folder(&gtk4::gio::File::for_path(&sys_dir));
        }
    }

    let filter = FileFilter::new();
    filter.set_name(Some("Desktop files (*.desktop)"));
    filter.add_pattern("*.desktop");
    dialog.add_filter(&filter);

    let s = state.clone();
    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept
            && let Some(file) = d.file()
            && let Some(mut path) = file.path()
        {
            if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
                path.set_extension("desktop");
            }
            match DesktopWriter::write_to_path(&de, &path) {
                Ok(saved_path) => {
                    let mut st = s.borrow_mut();
                    st.selected_path = Some(saved_path.clone());
                    st.is_dirty = false;
                    on_success(saved_path);
                }
                Err(e) => dialogs::show_error(d, &e.to_string()),
            }
        }
        d.close();
    });
    dialog.show();
}

pub fn do_save(
    widgets: &EntryWidgets,
    state: &SharedState,
    win: &impl IsA<gtk4::Window>,
    status_label: &gtk4::Label,
    refresh_list: impl Fn() + 'static,
) {
    match save_entry(widgets, state) {
        Ok((path, updated)) => {
            status_label.set_text(&path.to_string_lossy());
            refresh_list();
            dialogs::show_save_success(win, path, updated);
        }
        Err(e) => {
            status_label.set_text(&format!("Save failed: {}", e));
            dialogs::show_error(win, &e);
        }
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
        if DesktopReader::is_system_path(&path) {
            dialogs::show_system_file_info(
                win,
                "System application files located in /usr/share/applications (or other system directories) are read-only and managed by the system package manager.\n\nTo hide this launcher from application menus, check 'NoDisplay' (Hide from menus) and click 'Save .desktop'. This will create a user override that hides it.",
            );
            return;
        }

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
                {
                    let mut st = s.borrow_mut();
                    st.selected_path = None;
                    st.is_dirty = false;
                }
                refresh_list();
                lbl.set_text("Deleted");
            }
        });
    } else {
        dialogs::show_error(win, "No file selected to delete");
    }
}
