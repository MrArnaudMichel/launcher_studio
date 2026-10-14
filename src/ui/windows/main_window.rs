use crate::ui::editor::entry_form::{self};
use crate::ui::state;
use crate::ui::theme;
use crate::ui::windows::{actions, list_manager};
use adw::{
    ApplicationWindow as AdwApplicationWindow, Breakpoint, BreakpointCondition, Clamp,
    HeaderBar as AdwHeaderBar, NavigationPage, NavigationSplitView, ToolbarView, prelude::*,
};
use gtk4::{Align, Application, Box as GtkBox, Button, Orientation, ScrolledWindow};
use std::rc::Rc;

// Below this width the editor form puts labels above fields.
const COMPACT_FORM_CONDITION: &str = "max-width: 860sp";
// Below this width the sidebar and editor become stacked pages (phones).
const COLLAPSED_CONDITION: &str = "max-width: 600sp";

pub fn show_main_window(app: &impl IsA<Application>) {
    let app: Application = app.upcast_ref::<Application>().clone();
    let win = AdwApplicationWindow::builder()
        .application(&app)
        .title("Launcher Studio")
        .default_width(1050)
        .default_height(720)
        .width_request(360)
        .height_request(294)
        .resizable(true)
        .build();
    setup_css();

    let menu_button = crate::ui::components::menu_bar::build_primary_menu_button();
    let toolbar_data = crate::ui::components::toolbar::build_toolbar();
    let sidebar_data = crate::ui::components::sidebar::build_sidebar();
    let list_mgr = list_manager::LauncherListManager::new(sidebar_data);
    let status_data = crate::ui::components::status_bar::build_status_bar();
    let editor = entry_form::build_editor();
    let state = state::new_state();

    let clamp = Clamp::builder()
        .maximum_size(900)
        .tightening_threshold(600)
        .child(&editor.notebook)
        .build();
    let scroller = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&clamp)
        .build();

    {
        let st = state.clone();
        entry_form::wire_source_sync(&editor, move || {
            st.borrow_mut().is_dirty = true;
        });
    }
    state.borrow_mut().is_dirty = false;

    let buttons = build_action_buttons();

    // Sidebar page: launcher list with file actions and the primary menu.
    let sidebar_header = AdwHeaderBar::new();
    sidebar_header.pack_start(&toolbar_data.btn_new);
    sidebar_header.pack_start(&toolbar_data.btn_open);
    sidebar_header.pack_end(&menu_button);
    sidebar_header.pack_end(&toolbar_data.btn_refresh);
    let sidebar_view = ToolbarView::new();
    sidebar_view.add_top_bar(&sidebar_header);
    sidebar_view.set_content(Some(&list_mgr.sidebar.container));
    let sidebar_page = NavigationPage::builder()
        .title("Launcher Studio")
        .tag("sidebar")
        .child(&sidebar_view)
        .build();

    // Content page: editor with save/preview/delete and status in a bottom bar.
    let content_header = AdwHeaderBar::new();
    content_header.pack_end(&theme::create_theme_button());
    let bottom_bar = GtkBox::new(Orientation::Vertical, 6);
    bottom_bar.set_margin_top(6);
    bottom_bar.set_margin_bottom(6);
    bottom_bar.set_margin_start(12);
    bottom_bar.set_margin_end(12);
    bottom_bar.append(&buttons.container);
    bottom_bar.append(&status_data.container);
    let content_view = ToolbarView::new();
    content_view.add_top_bar(&content_header);
    content_view.set_content(Some(&scroller));
    content_view.add_bottom_bar(&bottom_bar);
    let content_page = NavigationPage::builder()
        .title("Editor")
        .tag("editor")
        .child(&content_view)
        .build();

    let split_view = NavigationSplitView::builder()
        .sidebar(&sidebar_page)
        .content(&content_page)
        .min_sidebar_width(280.0)
        .max_sidebar_width(380.0)
        .build();
    win.set_content(Some(&split_view));

    let compact_form = Breakpoint::new(
        BreakpointCondition::parse(COMPACT_FORM_CONDITION).expect("valid breakpoint condition"),
    );
    editor.compact.add_setters(&compact_form);
    win.add_breakpoint(compact_form);

    // Added last so it wins when both conditions match.
    let collapsed = Breakpoint::new(
        BreakpointCondition::parse(COLLAPSED_CONDITION).expect("valid breakpoint condition"),
    );
    collapsed.add_setter(&split_view, "collapsed", Some(&true.to_value()));
    editor.compact.add_setters(&collapsed);
    win.add_breakpoint(collapsed);

    let widgets = editor.widgets.clone();
    let listbox = list_mgr.sidebar.listbox.clone();
    let status_label = status_data.label.clone();

    let ensure_temp_row = {
        let lb = listbox.clone();
        let st = state.clone();
        let ne = widgets.name_entry.clone();
        let ie = widgets.icon_entry.clone();
        Rc::new(move || list_manager::create_temp_row(&lb, &st, &ne, &ie))
    };

    let refresh_list = {
        let mgr = list_mgr.clone();
        let st = state.clone();
        let sl = status_label.clone();
        let etr = ensure_temp_row.clone();
        Rc::new(move || mgr.refresh_list(&st, &sl, &*etr))
    };

    // Register focus search action on window
    {
        let focus_search_action = gtk4::gio::SimpleAction::new("focus_search", None);
        let se = list_mgr.sidebar.search_entry.clone();
        focus_search_action.connect_activate(move |_, _| {
            se.grab_focus();
        });
        win.add_action(&focus_search_action);
        app.set_accels_for_action("win.focus_search", &["<Ctrl>F"]);
    }

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
    connect_show_editor(&app, &listbox, &split_view);
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
    // Intentionally left empty: use GNOME default design system (Adwaita).
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
    let rl = refresh_list.clone();
    toolbar.btn_refresh.connect_clicked(move |_| rl());
}
// When collapsed, only one page is visible: jump to the editor once something is loaded.
fn connect_show_editor(
    app: &Application,
    listbox: &gtk4::ListBox,
    split_view: &NavigationSplitView,
) {
    let sv = split_view.clone();
    listbox.connect_row_activated(move |_, _| sv.set_show_content(true));
    for name in ["new", "open", "import_system"] {
        if let Some(action) = app
            .lookup_action(name)
            .and_downcast::<gtk4::gio::SimpleAction>()
        {
            let sv = split_view.clone();
            action.connect_activate(move |_, _| sv.set_show_content(true));
        }
    }
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
    buttons.delete.connect_clicked({
        let sl = sl.clone();
        let rl = rl.clone();
        move |_| {
            actions::do_delete(&s, &w, &wn, &sl, {
                let r = rl.clone();
                move || r()
            });
        }
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
        actions::do_save(&w, &s, &wn, &sl, {
            let r = rl.clone();
            move || r()
        });
    });
}
