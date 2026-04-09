use crate::ui::editor::entry_form::{self};
use crate::ui::state;
use crate::ui::theme;
use crate::ui::windows::{actions, list_manager};
use adw::{
    ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar, ToolbarView, prelude::*,
};
use gtk4::{Align, Application, Box as GtkBox, Button, Orientation, ScrolledWindow};
use std::rc::Rc;
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
    let state = state::new_state();
    let scroller = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    scroller.set_child(Some(&editor.notebook));
    {
        let st = state.clone();
        entry_form::wire_source_sync(&editor, move || {
            st.borrow_mut().is_dirty = true;
        });
    }
    state.borrow_mut().is_dirty = false;
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
        &app,
        &win,
        &widgets,
        state.clone(),
        &status_label,
        {
            let etr = ensure_temp_row.clone();
            move || etr()
        },
        {
            let rl = refresh_list.clone();
            move || rl()
        },
    );
    connect_toolbar_buttons(&app, &toolbar_data, refresh_list.clone());
    connect_sidebar(&listbox, &widgets, state.clone(), &status_label);
    connect_action_buttons(
        &buttons,
        &widgets,
        state.clone(),
        &win,
        &status_label,
        refresh_list.clone(),
    );
    refresh_list();
    win.present();
}
fn setup_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "scrolledwindow.frame { border-radius: 8px; }\n\
         textview { border-radius: 6px; }\n\
         entry { border-radius: 6px; }\n",
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
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
    ActionButtons {
        container,
        delete,
        preview,
        save,
    }
}
fn connect_toolbar_buttons(
    app: &Application,
    toolbar: &crate::ui::components::toolbar::Toolbar,
    refresh_list: Rc<dyn Fn()>,
) {
    let a = app.clone();
    toolbar.btn_new.connect_clicked(move |_| {
        a.activate_action("new", None);
    });
    let a = app.clone();
    toolbar.btn_open.connect_clicked(move |_| {
        a.activate_action("open", None);
    });
    let a = app.clone();
    toolbar.btn_save.connect_clicked(move |_| {
        a.activate_action("save", None);
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
