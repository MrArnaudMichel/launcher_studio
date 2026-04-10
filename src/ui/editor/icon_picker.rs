use crate::services::lucide_service::{LucideRenderSettings, LucideService};
use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

pub struct IconPickerDialog {
    dialog: gtk::Dialog,
    search_entry: gtk::SearchEntry,
    color_entry: gtk::Entry,
    stroke_spin: gtk::SpinButton,
    size_spin: gtk::SpinButton,
    icon_grid: gtk::FlowBox,
    status_label: gtk::Label,
    preview_image: gtk::Image,
    selected_icon: Rc<RefCell<Option<String>>>,
    lucide_service: Option<Rc<LucideService>>,
}

impl IconPickerDialog {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        let dialog = gtk::Dialog::builder()
            .title("Select Icon (Lucide)")
            .modal(true)
            .default_width(900)
            .default_height(700)
            .build();

        if let Some(p) = parent {
            dialog.set_transient_for(Some(p));
        }

        dialog.add_buttons(&[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Use this icon", gtk::ResponseType::Accept),
        ]);

        let content_area = dialog.content_area();
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        main_box.set_margin_top(10);
        main_box.set_margin_bottom(10);
        main_box.set_margin_start(10);
        main_box.set_margin_end(10);
        content_area.append(&main_box);

        // Header
        let help = gtk::Label::new(Some(
            "Search and select Lucide icons. Customize the style below.",
        ));
        help.set_xalign(0.0);
        help.add_css_class("title-4");
        main_box.append(&help);

        // Search entry
        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search icons (e.g., rocket, folder, terminal)..."));
        search_entry.set_hexpand(true);
        main_box.append(&search_entry);

        // Main content: icons grid + preview panel
        let content_paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        content_paned.set_vexpand(true);
        content_paned.set_hexpand(true);
        main_box.append(&content_paned);

        // Left: Icons grid
        let icons_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let grid_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .build();
        icons_box.append(&grid_scroll);

        let icon_grid = gtk::FlowBox::new();
        icon_grid.set_selection_mode(gtk::SelectionMode::Single);
        icon_grid.set_min_children_per_line(3);
        icon_grid.set_max_children_per_line(8);
        icon_grid.set_column_spacing(12);
        icon_grid.set_row_spacing(12);
        icon_grid.set_homogeneous(true);
        icon_grid.add_css_class("icon-grid");
        grid_scroll.set_child(Some(&icon_grid));

        let status_label = gtk::Label::new(Some("Loading icons..."));
        status_label.set_xalign(0.0);
        icons_box.append(&status_label);
        content_paned.set_start_child(Some(&icons_box));
        content_paned.set_position(600);

        // Right: Customization panel
        let settings_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        settings_box.set_margin_start(12);
        settings_box.set_margin_end(0);

        // Preview section
        let preview_label = gtk::Label::new(Some("Preview"));
        preview_label.add_css_class("heading");
        preview_label.set_xalign(0.0);
        settings_box.append(&preview_label);

        let preview_container = gtk::Box::new(gtk::Orientation::Vertical, 8);
        preview_container.set_halign(gtk::Align::Center);

        let preview_image = gtk::Image::new();
        preview_image.set_icon_size(gtk::IconSize::Large);
        preview_container.append(&preview_image);

        settings_box.append(&preview_container);

        // Settings section
        let settings_label = gtk::Label::new(Some("Style Settings"));
        settings_label.add_css_class("heading");
        settings_label.set_xalign(0.0);
        settings_box.append(&settings_label);

        // Color
        let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let color_lbl = gtk::Label::new(Some("Color"));
        color_lbl.set_width_chars(12);
        color_lbl.set_xalign(0.0);
        let color_entry = gtk::Entry::new();
        color_entry.set_text("#ffffff");
        color_entry.set_placeholder_text(Some("#ffffff"));
        color_entry.set_hexpand(true);
        color_row.append(&color_lbl);
        color_row.append(&color_entry);
        settings_box.append(&color_row);

        // Stroke width
        let stroke_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let stroke_lbl = gtk::Label::new(Some("Stroke Width"));
        stroke_lbl.set_width_chars(12);
        stroke_lbl.set_xalign(0.0);
        let stroke_spin = gtk::SpinButton::with_range(0.5, 4.0, 0.1);
        stroke_spin.set_value(2.0);
        stroke_spin.set_digits(1);
        stroke_spin.set_hexpand(true);
        stroke_row.append(&stroke_lbl);
        stroke_row.append(&stroke_spin);
        settings_box.append(&stroke_row);

        // Size
        let size_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let size_lbl = gtk::Label::new(Some("Size (px)"));
        size_lbl.set_width_chars(12);
        size_lbl.set_xalign(0.0);
        let size_spin = gtk::SpinButton::with_range(12.0, 256.0, 4.0);
        size_spin.set_value(48.0);
        size_spin.set_digits(0);
        size_spin.set_hexpand(true);
        size_row.append(&size_lbl);
        size_row.append(&size_spin);
        settings_box.append(&size_row);

        settings_box.set_vexpand(true);
        let settings_scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(false)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();
        settings_scroll.set_child(Some(&settings_box));
        content_paned.set_end_child(Some(&settings_scroll));

        let selected_icon = Rc::new(RefCell::new(None));
        let lucide_service = LucideService::new().ok().map(Rc::new);

        let this = Self {
            dialog,
            search_entry,
            color_entry,
            stroke_spin,
            size_spin,
            icon_grid,
            status_label,
            preview_image,
            selected_icon,
            lucide_service,
        };

        this.setup_events();
        this.refresh_results("");
        this
    }

    fn setup_events(&self) {
        let selected_icon = self.selected_icon.clone();
        let preview = self.preview_image.clone();
        let service = self.lucide_service.clone();
        let color_entry = self.color_entry.clone();
        let stroke_spin = self.stroke_spin.clone();
        let size_spin = self.size_spin.clone();

        // Handle child activation (double-click or Enter)
        self.icon_grid.connect_child_activated(move |_, child| {
            let icon_name = child.widget_name().to_string();
            *selected_icon.borrow_mut() = Some(icon_name.clone());

            if let Some(svc) = service.as_ref() {
                let settings = read_settings(&color_entry, &stroke_spin, &size_spin);
                if let Ok(path) = svc.preview_icon_svg(&icon_name, &settings) {
                    preview.set_from_file(Some(path.to_string_lossy().as_ref()));
                    preview.set_pixel_size(settings.size as i32);
                }
            }
        });

        // Handle selection changes (single-click)
        let selected_icon2 = self.selected_icon.clone();
        let preview2 = self.preview_image.clone();
        let service2 = self.lucide_service.clone();
        let color_entry2 = self.color_entry.clone();
        let stroke_spin2 = self.stroke_spin.clone();
        let size_spin2 = self.size_spin.clone();

        self.icon_grid.connect_selected_children_changed(move |flowbox| {
            if let Some(child) = flowbox.selected_children().first() {
                let icon_name = child.widget_name().to_string();
                *selected_icon2.borrow_mut() = Some(icon_name.clone());

                if let Some(svc) = service2.as_ref() {
                    let settings = read_settings(&color_entry2, &stroke_spin2, &size_spin2);
                    if let Ok(path) = svc.preview_icon_svg(&icon_name, &settings) {
                        preview2.set_from_file(Some(path.to_string_lossy().as_ref()));
                        preview2.set_pixel_size(settings.size as i32);
                    }
                }
            }
        });

        let this = self.clone_handles();
        self.search_entry.connect_search_changed(move |entry| {
            this.refresh_results(&entry.text());
        });

        let this = self.clone_handles();
        self.color_entry.connect_changed(move |_| {
            this.update_preview();
        });

        let this = self.clone_handles();
        self.stroke_spin.connect_value_changed(move |_| {
            this.update_preview();
        });

        let this = self.clone_handles();
        self.size_spin.connect_value_changed(move |_| {
            this.update_preview();
        });
    }

    fn update_preview(&self) {
        if let Some(icon_name) = self.selected_icon.borrow().as_ref() {
            if let Some(svc) = self.lucide_service.as_ref() {
                let settings = read_settings(&self.color_entry, &self.stroke_spin, &self.size_spin);
                if let Ok(path) = svc.preview_icon_svg(icon_name, &settings) {
                    self.preview_image.set_from_file(Some(path.to_string_lossy().as_ref()));
                    self.preview_image.set_pixel_size(settings.size as i32);
                }
            }
        }
    }

    pub fn run<F: FnOnce(Option<String>) + 'static>(self, callback: F) {
        let callback_cell = Rc::new(RefCell::new(Some(callback)));
        let selected_icon = self.selected_icon.clone();
        let status = self.status_label.clone();
        let service = self.lucide_service.clone();
        let color_entry = self.color_entry.clone();
        let stroke_spin = self.stroke_spin.clone();
        let size_spin = self.size_spin.clone();

        self.dialog.connect_response(move |d, resp| {
            if let Some(cb) = callback_cell.borrow_mut().take() {
                if resp == gtk::ResponseType::Accept {
                    if let Some(name) = selected_icon.borrow().clone() {
                        if let Some(svc) = service.as_ref() {
                            let settings = read_settings(&color_entry, &stroke_spin, &size_spin);
                            match svc.download_icon_svg_with_settings(&name, &settings) {
                                Ok(path) => cb(Some(path.to_string_lossy().to_string())),
                                Err(e) => {
                                    status.set_text(&format!("Download failed: {}", e));
                                    cb(None);
                                }
                            }
                        } else {
                            status.set_text("Lucide service is unavailable");
                            cb(None);
                        }
                    } else {
                        status.set_text("No icon selected");
                        cb(None);
                    }
                } else {
                    cb(None);
                }
            }
            d.close();
        });

        self.dialog.show();
    }

    fn refresh_results(&self, query: &str) {
        let settings = read_settings(&self.color_entry, &self.stroke_spin, &self.size_spin);

        let icons = if let Some(service) = &self.lucide_service {
            match service.search_icons(query, 120) {
                Ok(values) => values,
                Err(e) => {
                    self.status_label
                        .set_text(&format!("Lucide search failed: {}", e));
                    Vec::new()
                }
            }
        } else {
            self.status_label
                .set_text("Lucide service is unavailable on this system");
            Vec::new()
        };

        if let Some(service) = &self.lucide_service {
            populate_grid(&self.icon_grid, service, &icons, &settings);
        } else {
            clear_grid(&self.icon_grid);
        }

        self.status_label.set_text(&format!(
            "{} icon(s) found",
            icons.len(),
        ));

        // Auto-select first icon and show preview
        if let Some(first_child) = self.icon_grid.first_child() {
            if let Some(row) = first_child.downcast_ref::<gtk::FlowBoxChild>() {
                self.icon_grid.select_child(row);
                if let Some(icon_name) = icons.first() {
                    *self.selected_icon.borrow_mut() = Some(icon_name.clone());
                    if let Some(svc) = self.lucide_service.as_ref() {
                        if let Ok(path) = svc.preview_icon_svg(icon_name, &settings) {
                            self.preview_image.set_from_file(Some(path.to_string_lossy().as_ref()));
                            self.preview_image.set_pixel_size(settings.size as i32);
                        }
                    }
                }
            }
        }
    }

    fn clone_handles(&self) -> Self {
        Self {
            dialog: self.dialog.clone(),
            search_entry: self.search_entry.clone(),
            color_entry: self.color_entry.clone(),
            stroke_spin: self.stroke_spin.clone(),
            size_spin: self.size_spin.clone(),
            icon_grid: self.icon_grid.clone(),
            status_label: self.status_label.clone(),
            preview_image: self.preview_image.clone(),
            selected_icon: self.selected_icon.clone(),
            lucide_service: self.lucide_service.clone(),
        }
    }
}

fn read_settings(
    color_entry: &gtk::Entry,
    stroke_spin: &gtk::SpinButton,
    size_spin: &gtk::SpinButton,
) -> LucideRenderSettings {
    LucideRenderSettings {
        color_hex: color_entry.text().to_string(),
        stroke_width: stroke_spin.value(),
        size: size_spin.value_as_int().max(12) as u32,
    }
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

fn populate_grid(
    grid: &gtk::FlowBox,
    service: &LucideService,
    icons: &[String],
    settings: &LucideRenderSettings,
) {
    clear_grid(grid);

    for icon_name in icons {
        let item_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        item_box.set_margin_top(8);
        item_box.set_margin_bottom(8);
        item_box.set_margin_start(8);
        item_box.set_margin_end(8);
        item_box.set_halign(gtk::Align::Center);
        item_box.set_valign(gtk::Align::Center);

        // Icon image (56px for grid display)
        match service.preview_icon_svg(icon_name, &LucideRenderSettings {
            color_hex: settings.color_hex.clone(),
            stroke_width: settings.stroke_width,
            size: 56,
        }) {
            Ok(path) => {
                let image = gtk::Image::new();
                image.set_from_file(Some(path.to_string_lossy().as_ref()));
                image.set_pixel_size(56);
                item_box.append(&image);
            }
            Err(_) => {
                let image = gtk::Image::from_icon_name("image-missing");
                image.set_pixel_size(40);
                item_box.append(&image);
            }
        }

        // Label with tooltip
        let label = gtk::Label::new(Some(icon_name));
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_max_width_chars(10);
        label.set_wrap(true);
        label.set_xalign(0.5);
        label.add_css_class("caption");
        label.set_tooltip_text(Some(icon_name));
        item_box.append(&label);

        let child = gtk::FlowBoxChild::new();
        child.set_widget_name(icon_name);
        child.set_child(Some(&item_box));
        child.add_css_class("icon-grid-item");
        grid.insert(&child, -1);
    }

    grid.show();
}
