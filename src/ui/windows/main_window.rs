use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, FileChooserAction, FileChooserDialog, Orientation, ResponseType, ScrolledWindow};
use adw::{ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar, ToolbarView, prelude::*};
use std::rc::Rc;
use crate::domain::desktop_entry::DesktopEntry;
use crate::services::desktop_reader::DesktopReader;
use crate::ui::state;
use crate::ui::theme;
use crate::ui::editor::entry_form::{self, set_form_from_entry};
use crate::ui::windows::{actions, list_manager};
pub fn show_main_window(app: &impl IsA<Application>) {
    let app: Application = app.upcast_ref::<Application>().clone();
    let win = AdwApplicationWindow::builder()
        .application(&app)
        .title("Launcher Studio")
        .default_width(1000)
        .default_height(700)
        .resizable(true)
        .build();
    setup_css();
    let header = AdwHeaderBar::new();
    header.pack_end(&theme::create_theme_button());
    let root = GtkBox::new(Orientation::Vertical, 6);
    root.set_margin_top(0);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);
    let menubar = crate::ui::components::menu_bar::build_menu_bar(&app);
    let toolbar_data = crate::ui::components::toolbar::build_toolbar();
    let sidebar_data = crate::ui::components::sidebar::build_sidebar();
    let status_data = crate::ui::components::status_bar::build_status_bar();
    let editor = entry_form::build_editor();
    let scroller = ScrolledWindow::builder().hexpand(true).vexpand(true).build();
    scroller.set_child(Some(&editor.notebook));
    entry_form::wire_source_sync(&editor);
    let buttons = build_action_buttons();
    let main_area = GtkBox::new(Orientation::Horizontal, 12);
    main_area.append(&sidebar_data.container);
    main_area.append(&scroller);
    root.append(&menubar);
    root.append(&toolbar_data.container);
    root.append(&main_area);
    root.append(&buttons.container);
    root.append(&status_data.container);
    let toolbar_view = ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&root));
    win.set_content(Some(&toolbar_view));
    let state = state::new_state();
    let widgets = editor.widgets.clone();
    let listbox = sidebar_data.listbox.clone();
    let status_label = status_data.label.clone();
    let ensure_temp_row = {
        let lb = listbox.clone();
        let st = state.clone();
        let ne = widgets.name_entry.clone();
        let ie = widgets.icon_entry.clone();
        Rc::new(move || list_manager::create_temp_row(&lb, &st, &ne, &ie))
    };
    let refresh_list = {
        let lb = listbox.clone();
        let st = state.clone();
        let sl = status_label.clone();
        let etr = ensure_temp_row.clone();
        Rc::new(move || list_manager::refresh_desktop_list(&lb, &st, &sl, &*etr))
    };
    actions::register_actions(
        &app, &win, &widgets, state.clone(), &status_label,
        {
            let etr = ensure_temp_row.clone();
            move || etr()
        },
        {
            let rl = refresh_list.clone();
            move || rl()
        },
    );
    connect_toolbar_buttons(
        &toolbar_data, &widgets, state.clone(), &status_label,
        ensure_temp_row.clone(), refresh_list.clone(),
    );
    connect_sidebar(&listbox, &widgets, state.clone(), &status_label);
    connect_action_buttons(
        &buttons, &widgets, state.clone(), &win, &status_label, refresh_list.clone(),
    );
    refresh_list();
    win.present();
}
fn setup_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "scrolledwindow.frame { border-radius: 8px; }\n\
         textview { border-radius: 6px; }\n\
         entry { border-radius: 6px; }\n"
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
        );
    }
}
struct ActionButtons {
    container: GtkBox,
    delete: Button,
    preview: Button,
    save: Button,
}
fn build_action_buttons() -> ActionButtons {
    let container = GtkBox::new(Orientation::Horizontal, 8);
    container.set_halign(Align::End);
    let delete = Button::with_label("Delete");
    delete.add_css_class("destructive-action");
    let preview = Button::with_label("Preview");
    let save = Button::with_label("Save .desktop");
    container.append(&delete);
    container.append(&preview);
    container.append(&save);
    ActionButtons { container, delete, preview, save }
}
fn connect_toolbar_buttons(
    toolbar: &crate::ui::components::toolbar::Toolbar,
    widgets: &entry_form::EntryWidgets,
    state: state::SharedState,
    status_label: &gtk4::Label,
    ensure_temp_row: Rc<dyn Fn()>,
    refresh_list: Rc<dyn Fn()>,
) {
    let w = widgets.clone();
    let s = state.clone();
    let sl = status_label.clone();
    let etr = ensure_temp_row.clone();
    toolbar.btn_new.connect_clicked(move |_| {
        set_form_from_entry(&w, &DesktopEntry::default());
        {
            let mut st = s.borrow_mut();
            st.selected_path = None;
            st.in_edit = true;
        }
        etr();
        w.type_combo.set_sensitive(true);
        sl.set_text("New entry");
    });
    let w = widgets.clone();
    let s = state.clone();
    let sl = status_label.clone();
    toolbar.btn_open.connect_clicked(move |_| {
        let dialog = FileChooserDialog::new(
            Some("Open .desktop"), None::<&ApplicationWindow>, FileChooserAction::Open,
            &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
        );
        let w2 = w.clone();
        let s2 = s.clone();
        let sl2 = sl.clone();
        dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        match DesktopReader::read_from_path(&path) {
                            Ok(de) => {
                                set_form_from_entry(&w2, &de);
                                w2.type_combo.set_sensitive(false);
                                s2.borrow_mut().selected_path = Some(path.clone());
                                sl2.set_text(&path.to_string_lossy());
                            }
                            Err(e) => sl2.set_text(&format!("Open failed: {}", e)),
                        }
                    }
                }
            }
            d.close();
        });
        dialog.show();
    });
    let w = widgets.clone();
    let s = state.clone();
    let sl = status_label.clone();
    toolbar.btn_save.connect_clicked(move |_| {
        match entry_form::collect_entry(&w) {
            Ok(de) => {
                let sel = s.borrow().selected_path.clone();
                if let Some(path) = sel {
                    match crate::services::desktop_writer::DesktopWriter::write_to_path(&de, &path) {
                        Ok(_) => sl.set_text(&format!("Updated: {}", path.display())),
                        Err(e) => sl.set_text(&format!("Save failed: {}", e)),
                    }
                } else {
                    let fname = if !de.name.trim().is_empty() { de.name.clone() } else { "desktop-entry".into() };
                    match crate::services::desktop_writer::DesktopWriter::write(&de, &fname, true) {
                        Ok(p) => sl.set_text(&format!("Saved: {}", p.display())),
                        Err(e) => sl.set_text(&format!("Save failed: {}", e)),
                    }
                }
            }
            Err(e) => sl.set_text(&format!("Invalid: {}", e)),
        }
    });
    let rl = refresh_list.clone();
    toolbar.btn_refresh.connect_clicked(move |_| rl());
}
fn connect_sidebar(
    listbox: &gtk4::ListBox,
    widgets: &entry_form::EntryWidgets,
    state: state::SharedState,
    status_label: &gtk4::Label,
) {
    let w = widgets.clone();
    let s = state.clone();
    let sl = status_label.clone();
    let lb = listbox.clone();
    listbox.connect_row_activated(move |_, row| {
        list_manager::on_row_activated(row, &s, &w, &sl, &lb);
    });
}
fn connect_action_buttons(
    buttons: &ActionButtons,
    widgets: &entry_form::EntryWidgets,
    state: state::SharedState,
    win: &AdwApplicationWindow,
    status_label: &gtk4::Label,
    refresh_list: Rc<dyn Fn()>,
) {
    let w = widgets.clone();
    let s = state.clone();
    let wn = win.clone();
    let sl = status_label.clone();
    let rl = refresh_list.clone();
    buttons.delete.connect_clicked(move |_| {
        actions::do_delete(&s, &w, &wn, &sl, {
            let r = rl.clone();
            move || r()
        });
    });
    let w = widgets.clone();
    let wn = win.clone();
    buttons.preview.connect_clicked(move |_| {
        actions::do_preview(&w, &wn);
    });
    let w = widgets.clone();
    let s = state.clone();
    let wn = win.clone();
    buttons.save.connect_clicked(move |_| {
        actions::do_save(&w, &s, &wn);
    });
}
