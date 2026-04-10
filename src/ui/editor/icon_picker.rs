use crate::services::lucide_service::{LucideRenderSettings, LucideService};
use adw::{
    HeaderBar as AdwHeaderBar, ToolbarView as AdwToolbarView, Window as AdwWindow,
    WindowTitle as AdwWindowTitle, prelude::*,
};
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_RENDER_SIZE: u32 = 128;
const PREVIEW_PIXEL_SIZE: i32 = 144;
const GRID_PIXEL_SIZE: i32 = 64;
const SEARCH_LIMIT: usize = 72;
const DEFAULT_COLOR: &str = "#ffffff";
const DEFAULT_STROKE_WIDTH: f64 = 2.0;
const STACK_RESULTS: &str = "results";
const STACK_STATE: &str = "state";
const CONTENT_PADDING: i32 = 16;
const SECTION_SPACING: i32 = 12;

pub struct IconPickerDialog {
    window: AdwWindow,
    network_label: gtk::Label,
    retry_button: gtk::Button,
    search_entry: gtk::SearchEntry,
    color_entry: gtk::Entry,
    stroke_spin: gtk::SpinButton,
    icon_grid: gtk::FlowBox,
    status_label: gtk::Label,
    result_stack: gtk::Stack,
    state_icon: gtk::Image,
    state_title: gtk::Label,
    state_description: gtk::Label,
    preview_image: gtk::Image,
    preview_name: gtk::Label,
    use_button: gtk::Button,
    cancel_button: gtk::Button,
    selected_icon: Rc<RefCell<Option<String>>>,
    lucide_service: Option<Rc<LucideService>>,
}

impl IconPickerDialog {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        let window = AdwWindow::builder()
            .title("Lucide icons")
            .modal(true)
            .default_width(1120)
            .default_height(760)
            .resizable(true)
            .build();

        if let Some(p) = parent {
            window.set_transient_for(Some(p));
            if let Some(app) = p.application() {
                window.set_application(Some(&app));
            }
        }

        let header = AdwHeaderBar::new();
        let header_title = AdwWindowTitle::new("Lucide Icon Picker", "GNOME style integration");
        header.set_title_widget(Some(&header_title));

        let cancel_button = gtk::Button::with_label("Cancel");
        let use_button = gtk::Button::with_label("Use this icon");
        use_button.add_css_class("suggested-action");
        use_button.set_sensitive(false);

        let toolbar_view = AdwToolbarView::new();
        toolbar_view.add_top_bar(&header);

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);
        main_box.set_margin_top(CONTENT_PADDING);
        main_box.set_margin_bottom(CONTENT_PADDING);
        main_box.set_margin_start(CONTENT_PADDING);
        main_box.set_margin_end(CONTENT_PADDING);

        let title_label = gtk::Label::new(Some(
            "Search Lucide icons, preview them, then tune color and stroke width.",
        ));
        title_label.set_wrap(true);
        title_label.set_xalign(0.0);
        title_label.add_css_class("title-4");
        main_box.append(&title_label);

        let network_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let network_label = gtk::Label::new(Some("Checking internet connection..."));
        network_label.set_xalign(0.0);
        network_label.set_hexpand(true);
        network_label.add_css_class("dim-label");
        let retry_button = gtk::Button::with_label("Retry");
        retry_button.add_css_class("flat");
        network_row.append(&network_label);
        network_row.append(&retry_button);
        main_box.append(&network_row);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some(
            "Search icons (rocket, folder, terminal, app-window...)",
        ));
        search_entry.set_hexpand(true);
        main_box.append(&search_entry);

        let content_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        content_paned.set_hexpand(true);
        content_paned.set_vexpand(true);
        content_paned.set_position(650);
        main_box.append(&content_paned);

        let actions_row = gtk::Box::new(gtk::Orientation::Horizontal, SECTION_SPACING);
        actions_row.set_halign(gtk::Align::Center);
        actions_row.append(&cancel_button);
        actions_row.append(&use_button);
        main_box.append(&actions_row);

        let icon_grid = gtk::FlowBox::new();
        icon_grid.set_selection_mode(gtk::SelectionMode::Single);
        icon_grid.set_activate_on_single_click(true);
        icon_grid.set_min_children_per_line(4);
        icon_grid.set_max_children_per_line(6);
        icon_grid.set_column_spacing(10);
        icon_grid.set_row_spacing(10);
        icon_grid.set_homogeneous(true);
        icon_grid.set_valign(gtk::Align::Start);

        let grid_scroll = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();
        grid_scroll.set_child(Some(&icon_grid));

        let status_label = gtk::Label::new(Some("Loading icons..."));
        status_label.set_xalign(0.0);
        status_label.add_css_class("dim-label");

        let results_page = gtk::Box::new(gtk::Orientation::Vertical, 8);
        results_page.append(&grid_scroll);
        results_page.append(&status_label);

        let state_page = gtk::Box::new(gtk::Orientation::Vertical, 8);
        state_page.set_halign(gtk::Align::Center);
        state_page.set_valign(gtk::Align::Center);
        state_page.set_vexpand(true);

        let state_icon = gtk::Image::from_icon_name("network-offline-symbolic");
        state_icon.set_pixel_size(48);
        state_page.append(&state_icon);

        let state_title = gtk::Label::new(Some("No content"));
        state_title.add_css_class("title-4");
        state_title.set_wrap(true);
        state_title.set_justify(gtk::Justification::Center);
        state_page.append(&state_title);

        let state_description = gtk::Label::new(Some(""));
        state_description.add_css_class("dim-label");
        state_description.set_wrap(true);
        state_description.set_justify(gtk::Justification::Center);
        state_page.append(&state_description);

        let result_stack = gtk::Stack::new();
        result_stack.set_hexpand(true);
        result_stack.set_vexpand(true);
        result_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        result_stack.set_transition_duration(180);
        result_stack.add_named(&results_page, Some(STACK_RESULTS));
        result_stack.add_named(&state_page, Some(STACK_STATE));

        content_paned.set_start_child(Some(&result_stack));

        let settings_scroll = gtk::ScrolledWindow::builder()
            .hexpand(false)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_width(320)
            .build();

        let settings_box = gtk::Box::new(gtk::Orientation::Vertical, SECTION_SPACING);

        let preview_frame = gtk::Frame::new(None);
        preview_frame.set_label(Some("Preview"));

        let preview_stack = gtk::Box::new(gtk::Orientation::Vertical, 8);
        preview_stack.set_margin_top(CONTENT_PADDING);
        preview_stack.set_margin_bottom(CONTENT_PADDING);
        preview_stack.set_margin_start(CONTENT_PADDING);
        preview_stack.set_margin_end(CONTENT_PADDING);
        preview_stack.set_halign(gtk::Align::Center);

        let preview_image = gtk::Image::new();
        preview_image.set_pixel_size(PREVIEW_PIXEL_SIZE);
        preview_stack.append(&preview_image);

        let preview_name = gtk::Label::new(Some("No icon selected yet"));
        preview_name.set_wrap(true);
        preview_name.set_justify(gtk::Justification::Center);
        preview_stack.append(&preview_name);

        preview_frame.set_child(Some(&preview_stack));
        settings_box.append(&preview_frame);

        let style_frame = gtk::Frame::new(None);
        style_frame.set_label(Some("Style settings"));

        let style_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        style_box.set_margin_top(CONTENT_PADDING);
        style_box.set_margin_bottom(CONTENT_PADDING);
        style_box.set_margin_start(CONTENT_PADDING);
        style_box.set_margin_end(CONTENT_PADDING);

        let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let color_lbl = gtk::Label::new(Some("Color"));
        color_lbl.set_width_chars(12);
        color_lbl.set_xalign(0.0);
        let color_entry = gtk::Entry::new();
        color_entry.set_text(DEFAULT_COLOR);
        color_entry.set_placeholder_text(Some(DEFAULT_COLOR));
        color_entry.set_hexpand(true);
        color_row.append(&color_lbl);
        color_row.append(&color_entry);
        style_box.append(&color_row);

        let stroke_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let stroke_lbl = gtk::Label::new(Some("Stroke width"));
        stroke_lbl.set_width_chars(12);
        stroke_lbl.set_xalign(0.0);
        let stroke_spin = gtk::SpinButton::with_range(0.5, 4.0, 0.1);
        stroke_spin.set_value(DEFAULT_STROKE_WIDTH);
        stroke_spin.set_digits(1);
        stroke_spin.set_hexpand(true);
        stroke_row.append(&stroke_lbl);
        stroke_row.append(&stroke_spin);
        style_box.append(&stroke_row);

        let tips = gtk::Label::new(Some(
            "Tip: search, pick an icon, then adjust color and stroke width.",
        ));
        tips.add_css_class("dim-label");
        tips.set_wrap(true);
        tips.set_xalign(0.0);
        style_box.append(&tips);

        style_frame.set_child(Some(&style_box));
        settings_box.append(&style_frame);

        settings_scroll.set_child(Some(&settings_box));
        content_paned.set_end_child(Some(&settings_scroll));
        toolbar_view.set_content(Some(&main_box));
        window.set_content(Some(&toolbar_view));
        window.set_default_widget(Some(&use_button));

        let selected_icon = Rc::new(RefCell::new(None));
        let lucide_service = LucideService::new().ok().map(Rc::new);

        let this = Self {
            window,
            network_label,
            retry_button,
            search_entry,
            color_entry,
            stroke_spin,
            icon_grid,
            status_label,
            result_stack,
            state_icon,
            state_title,
            state_description,
            preview_image,
            preview_name,
            use_button,
            cancel_button,
            selected_icon,
            lucide_service,
        };

        this.setup_events();
        this.refresh_results("");
        this
    }

    fn setup_events(&self) {
        let this = self.clone_handles();
        self.retry_button.connect_clicked(move |_| {
            this.refresh_results(&this.search_entry.text());
        });

        let this = self.clone_handles();
        self.search_entry.connect_search_changed(move |entry| {
            this.refresh_results(&entry.text());
        });

        let this = self.clone_handles();
        self.color_entry.connect_changed(move |_| {
            this.update_preview();
            this.refresh_grid_icons();
        });

        let this = self.clone_handles();
        self.stroke_spin.connect_value_changed(move |_| {
            this.update_preview();
            this.refresh_grid_icons();
        });

        let this = self.clone_handles();
        self.icon_grid.connect_selected_children_changed(move |flowbox| {
            if let Some(child) = flowbox.selected_children().first() {
                let icon_name = child.widget_name().to_string();
                this.select_icon(&icon_name);
            } else {
                this.use_button.set_sensitive(false);
            }
        });

        let this = self.clone_handles();
        self.icon_grid.connect_child_activated(move |_, child| {
            let icon_name = child.widget_name().to_string();
            this.select_icon(&icon_name);
        });
    }

    fn select_icon(&self, icon_name: &str) {
        *self.selected_icon.borrow_mut() = Some(icon_name.to_string());
        self.preview_name.set_text(icon_name);
        self.use_button.set_sensitive(true);
        self.update_preview();
    }

    fn reset_preview(&self, message: &str) {
        self.preview_image.set_icon_name(Some("image-missing"));
        self.preview_image.set_pixel_size(PREVIEW_PIXEL_SIZE);
        self.preview_name.set_text(message);
        self.use_button.set_sensitive(false);
    }

    fn update_preview(&self) {
        let Some(icon_name) = self.selected_icon.borrow().clone() else {
            self.reset_preview("No icon selected yet");
            return;
        };

        let Some(svc) = self.lucide_service.as_ref() else {
            self.reset_preview("Lucide service unavailable on this system");
            return;
        };

        let settings = read_settings(&self.color_entry, &self.stroke_spin);
        if !is_valid_hex_color(&settings.color_hex) {
            self.reset_preview("Invalid color, use #RRGGBB or #RGB");
            return;
        }

        match svc.preview_icon_svg(&icon_name, &settings) {
            Ok(path) => {
                self.preview_image.set_from_file(Some(path.to_string_lossy().as_ref()));
                self.preview_image.set_pixel_size(PREVIEW_PIXEL_SIZE);
                self.preview_name.set_text(&icon_name);
                self.use_button.set_sensitive(true);
            }
            Err(err) => {
                self.reset_preview(&format!("Preview unavailable: {}", err));
            }
        }
    }

    fn refresh_grid_icons(&self) {
        if self.result_stack.visible_child_name().as_deref() != Some(STACK_RESULTS) {
            return;
        }
        self.refresh_results(&self.search_entry.text());
    }

    pub fn run<F: FnOnce(Option<String>) + 'static>(self, callback: F) {
        let callback_cell = Rc::new(RefCell::new(Some(callback)));
        let finish = {
            let callback_cell = callback_cell.clone();
            let window = self.window.clone();
            Rc::new(move |result: Option<String>| {
                if let Some(cb) = callback_cell.borrow_mut().take() {
                    cb(result);
                }
                window.close();
            })
        };

        {
            let finish = finish.clone();
            self.cancel_button.connect_clicked(move |_| {
                finish(None);
            });
        }

        {
            let finish = finish.clone();
            let selected_icon = self.selected_icon.clone();
            let service = self.lucide_service.clone();
            let color_entry = self.color_entry.clone();
            let stroke_spin = self.stroke_spin.clone();
            let status = self.status_label.clone();
            self.use_button.connect_clicked(move |_| {
                let Some(name) = selected_icon.borrow().clone() else {
                    status.set_text("No icon selected");
                    return;
                };

                let Some(svc) = service.as_ref() else {
                    status.set_text("Lucide service is unavailable");
                    return;
                };

                let settings = read_settings(&color_entry, &stroke_spin);
                if !is_valid_hex_color(&settings.color_hex) {
                    status.set_text("Invalid color, expected #RRGGBB or #RGB");
                    return;
                }

                match svc.download_icon_svg_with_settings(&name, &settings) {
                    Ok(path) => finish(Some(path.to_string_lossy().to_string())),
                    Err(err) => status.set_text(&format!("Download failed: {}", err)),
                }
            });
        }

        self.window.present();
    }

    fn refresh_results(&self, query: &str) {
        let online = self
            .lucide_service
            .as_ref()
            .map(|service| service.is_online())
            .unwrap_or(false);

        self.apply_online_state(online);

        if !online {
            clear_grid(&self.icon_grid);
            self.selected_icon.borrow_mut().take();
            self.reset_preview("Offline mode: connect to internet to use Lucide icons");
            self.status_label
                .set_text("Offline mode: Lucide search and download are unavailable");
            self.show_state(
                "network-offline-symbolic",
                "Offline",
                "Lucide icons require an internet connection.",
            );
            return;
        }

        let settings = read_settings(&self.color_entry, &self.stroke_spin);
        let current_selection = self.selected_icon.borrow().clone();

        let (icons, status_text) = if let Some(service) = &self.lucide_service {
            match service.search_icons(query, SEARCH_LIMIT) {
                Ok(values) => {
                    let count = values.len();
                    (values, format!("{} icon(s) found", count))
                }
                Err(err) => {
                    self.show_state(
                        "dialog-error-symbolic",
                        "Search unavailable",
                        &format!("Could not fetch Lucide results: {}", err),
                    );
                    clear_grid(&self.icon_grid);
                    self.selected_icon.borrow_mut().take();
                    self.reset_preview("No icon selected yet");
                    self.status_label
                        .set_text("Search failed. Check connection and retry.");
                    return;
                }
            }
        } else {
            clear_grid(&self.icon_grid);
            self.selected_icon.borrow_mut().take();
            self.reset_preview("Lucide service unavailable on this system");
            self.show_state(
                "dialog-error-symbolic",
                "Service unavailable",
                "The Lucide service could not be initialized.",
            );
            self.status_label
                .set_text("Lucide service is unavailable on this system");
            return;
        };

        if icons.is_empty() {
            clear_grid(&self.icon_grid);
            self.selected_icon.borrow_mut().take();
            self.reset_preview("No icon selected yet");
            self.status_label.set_text("No icon found for this query");
            self.show_state(
                "edit-find-symbolic",
                "No results",
                "Try another keyword, for example: folder, rocket, settings.",
            );
            return;
        }

        populate_grid(&self.icon_grid, self.lucide_service.as_deref(), &icons, &settings);
        self.status_label.set_text(&status_text);
        self.result_stack.set_visible_child_name(STACK_RESULTS);

        let next_selection = current_selection
            .filter(|selected| icons.iter().any(|name| name == selected))
            .or_else(|| icons.first().cloned());

        if let Some(icon_name) = next_selection {
            if select_child_by_name(&self.icon_grid, &icon_name) {
                self.select_icon(&icon_name);
            }
        } else {
            self.selected_icon.borrow_mut().take();
            self.reset_preview("No icon selected yet");
        }
    }

    fn clone_handles(&self) -> Self {
        Self {
            window: self.window.clone(),
            network_label: self.network_label.clone(),
            retry_button: self.retry_button.clone(),
            search_entry: self.search_entry.clone(),
            color_entry: self.color_entry.clone(),
            stroke_spin: self.stroke_spin.clone(),
            icon_grid: self.icon_grid.clone(),
            status_label: self.status_label.clone(),
            result_stack: self.result_stack.clone(),
            state_icon: self.state_icon.clone(),
            state_title: self.state_title.clone(),
            state_description: self.state_description.clone(),
            preview_image: self.preview_image.clone(),
            preview_name: self.preview_name.clone(),
            use_button: self.use_button.clone(),
            cancel_button: self.cancel_button.clone(),
            selected_icon: self.selected_icon.clone(),
            lucide_service: self.lucide_service.clone(),
        }
    }

    fn apply_online_state(&self, online: bool) {
        if online {
            self.network_label
                .set_text("Online mode: Lucide search and download available");
            self.search_entry.set_sensitive(true);
            self.icon_grid.set_sensitive(true);
            self.color_entry.set_sensitive(true);
            self.stroke_spin.set_sensitive(true);
        } else {
            self.network_label
                .set_text("Offline mode: Lucide requires internet connection");
            self.search_entry.set_sensitive(false);
            self.icon_grid.set_sensitive(false);
            self.color_entry.set_sensitive(false);
            self.stroke_spin.set_sensitive(false);
            self.use_button.set_sensitive(false);
        }
    }

    fn show_state(&self, icon_name: &str, title: &str, description: &str) {
        self.state_icon.set_icon_name(Some(icon_name));
        self.state_title.set_text(title);
        self.state_description.set_text(description);
        self.result_stack.set_visible_child_name(STACK_STATE);
    }
}

fn read_settings(color_entry: &gtk::Entry, stroke_spin: &gtk::SpinButton) -> LucideRenderSettings {
    LucideRenderSettings {
        color_hex: color_entry.text().to_string(),
        stroke_width: stroke_spin.value(),
        size: DEFAULT_RENDER_SIZE,
    }
}

fn is_valid_hex_color(input: &str) -> bool {
    let value = input.trim().trim_start_matches('#');
    (value.len() == 3 || value.len() == 6) && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn clear_grid(grid: &gtk::FlowBox) {
    let mut children = Vec::new();
    let mut current = grid.first_child();
    while let Some(widget) = current {
        current = widget.next_sibling();
        children.push(widget);
    }
    for child in children {
        grid.remove(&child);
    }
}

fn select_child_by_name(grid: &gtk::FlowBox, name: &str) -> bool {
    let mut current = grid.first_child();
    while let Some(widget) = current {
        current = widget.next_sibling();
        if let Some(child) = widget.downcast_ref::<gtk::FlowBoxChild>() {
            if child.widget_name() == name {
                grid.select_child(child);
                return true;
            }
        }
    }
    false
}

fn populate_grid(
    grid: &gtk::FlowBox,
    service: Option<&LucideService>,
    icons: &[String],
    settings: &LucideRenderSettings,
) {
    clear_grid(grid);

    let preview_settings = LucideRenderSettings {
        color_hex: settings.color_hex.clone(),
        stroke_width: settings.stroke_width,
        size: GRID_PIXEL_SIZE as u32,
    };

    for icon_name in icons {
        let tile = gtk::Box::new(gtk::Orientation::Vertical, 6);
        tile.set_halign(gtk::Align::Center);
        tile.set_valign(gtk::Align::Center);
        tile.set_size_request(112, 112);

        match service.and_then(|svc| svc.preview_icon_svg(icon_name, &preview_settings).ok()) {
            Some(path) => {
                let image = gtk::Image::new();
                image.set_from_file(Some(path.to_string_lossy().as_ref()));
                image.set_pixel_size(GRID_PIXEL_SIZE);
                image.set_halign(gtk::Align::Center);
                tile.append(&image);
            }
            None => {
                let image = gtk::Image::from_icon_name("image-missing");
                image.set_pixel_size(GRID_PIXEL_SIZE - 4);
                image.set_halign(gtk::Align::Center);
                tile.append(&image);
            }
        }

        let label = gtk::Label::new(Some(icon_name));
        label.set_wrap(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_xalign(0.5);
        label.set_max_width_chars(12);
        label.add_css_class("caption");
        tile.append(&label);

        let child = gtk::FlowBoxChild::new();
        child.set_widget_name(icon_name);
        child.set_child(Some(&tile));
        grid.insert(&child, -1);
    }

    grid.show();
}
