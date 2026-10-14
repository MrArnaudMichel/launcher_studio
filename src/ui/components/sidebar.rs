use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Image, Justification, Label, ListBox, Orientation,
    ScrolledWindow, SearchEntry, Stack, StackTransitionType, ToggleButton,
};

pub struct Sidebar {
    pub container: GtkBox,
    pub listbox: ListBox,
    pub search_entry: SearchEntry,
    pub btn_all: ToggleButton,
    pub btn_home: ToggleButton,
    pub btn_system: ToggleButton,
    pub btn_flatpak: ToggleButton,
    pub count_all: Label,
    pub count_home: Label,
    pub count_system: Label,
    pub count_flatpak: Label,
    pub count_label: Label,
    pub stack: Stack,
    pub clear_btn: Button,
}

pub fn build_sidebar() -> Sidebar {
    let container = GtkBox::new(Orientation::Vertical, 8);
    container.set_margin_top(6);
    container.set_margin_bottom(12);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_vexpand(true);

    // Search bar
    let search_entry = SearchEntry::new();
    search_entry.set_placeholder_text(Some("Search launchers (Ctrl+F)..."));
    search_entry.set_hexpand(true);
    container.append(&search_entry);

    // Filter bar (Segmented ToggleButtons: All | Home | /usr | Flatpak)
    let filter_bar = GtkBox::new(Orientation::Horizontal, 0);
    filter_bar.add_css_class("linked");
    filter_bar.set_hexpand(true);

    let (btn_all, count_all) = filter_button("All");
    btn_all.set_tooltip_text(Some("Show all launchers"));
    btn_all.set_hexpand(true);

    let (btn_home, count_home) = filter_button("Home");
    btn_home.set_tooltip_text(Some("Show user launchers (~/.local/share/applications)"));
    btn_home.set_group(Some(&btn_all));
    btn_home.set_hexpand(true);

    let (btn_system, count_system) = filter_button("/usr");
    btn_system.set_tooltip_text(Some("Show system launchers (/usr/share/applications)"));
    btn_system.set_group(Some(&btn_all));
    btn_system.set_hexpand(true);

    let (btn_flatpak, count_flatpak) = filter_button("Flatpak");
    btn_flatpak.set_tooltip_text(Some("Show Flatpak application launchers"));
    btn_flatpak.set_group(Some(&btn_all));
    btn_flatpak.set_hexpand(true);

    btn_all.set_active(true);

    filter_bar.append(&btn_all);
    filter_bar.append(&btn_home);
    filter_bar.append(&btn_system);
    filter_bar.append(&btn_flatpak);
    container.append(&filter_bar);

    // Summary / Count info row
    let count_label = Label::new(Some("Loading launchers..."));
    count_label.add_css_class("dim-label");
    count_label.add_css_class("caption");
    count_label.set_xalign(0.0);
    container.append(&count_label);

    // List & Empty state stack
    let stack = Stack::new();
    stack.set_transition_type(StackTransitionType::Crossfade);
    stack.set_transition_duration(150);
    stack.set_vexpand(true);
    stack.set_hexpand(true);

    let scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    let listbox = ListBox::new();
    listbox.add_css_class("navigation-sidebar");
    listbox.set_selection_mode(gtk4::SelectionMode::Single);
    scroller.set_child(Some(&listbox));

    // Empty state view
    let empty_box = GtkBox::new(Orientation::Vertical, 10);
    empty_box.set_valign(Align::Center);
    empty_box.set_halign(Align::Center);
    empty_box.set_vexpand(true);
    empty_box.set_hexpand(true);
    empty_box.set_margin_start(16);
    empty_box.set_margin_end(16);

    let empty_icon = Image::from_icon_name("system-search-symbolic");
    empty_icon.set_pixel_size(48);
    empty_icon.add_css_class("dim-label");
    empty_box.append(&empty_icon);

    let empty_title = Label::new(Some("No launchers found"));
    empty_title.add_css_class("title-4");
    empty_title.set_wrap(true);
    empty_title.set_justify(Justification::Center);
    empty_box.append(&empty_title);

    let empty_sub = Label::new(Some("Try adjusting your search query or location filter."));
    empty_sub.add_css_class("dim-label");
    empty_sub.add_css_class("caption");
    empty_sub.set_wrap(true);
    empty_sub.set_justify(Justification::Center);
    empty_box.append(&empty_sub);

    let clear_btn = Button::with_label("Clear Filters");
    clear_btn.add_css_class("pill");
    clear_btn.set_halign(Align::Center);
    empty_box.append(&clear_btn);

    stack.add_named(&scroller, Some("list"));
    stack.add_named(&empty_box, Some("empty"));
    stack.set_visible_child_name("list");

    container.append(&stack);

    // Handle Escape in search entry to clear search and focus list
    {
        let key_controller = gtk4::EventControllerKey::new();
        let se = search_entry.clone();
        let lb = listbox.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                if !se.text().is_empty() {
                    se.set_text("");
                }
                lb.grab_focus();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        });
        search_entry.add_controller(key_controller);
    }

    Sidebar {
        container,
        listbox,
        search_entry,
        btn_all,
        btn_home,
        btn_system,
        btn_flatpak,
        count_all,
        count_home,
        count_system,
        count_flatpak,
        count_label,
        stack,
        clear_btn,
    }
}

// Filter toggle with the location name above a small count, so four fit on a phone.
fn filter_button(name: &str) -> (ToggleButton, Label) {
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.append(&Label::new(Some(name)));
    let count = Label::new(Some("–"));
    count.add_css_class("caption");
    count.add_css_class("numeric");
    content.append(&count);
    let btn = ToggleButton::new();
    btn.set_child(Some(&content));
    (btn, count)
}
