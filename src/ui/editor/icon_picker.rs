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
    result_list: gtk::ListBox,
    status_label: gtk::Label,
    selected_icon: Rc<RefCell<Option<String>>>,
    lucide_service: Option<Rc<LucideService>>,
}

impl IconPickerDialog {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        let dialog = gtk::Dialog::builder()
            .title("Select Icon (Lucide)")
            .modal(true)
            .default_width(640)
            .default_height(520)
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

        let help = gtk::Label::new(Some(
            "Search Lucide icons, tweak style, and save the rendered SVG locally for Icon=...",
        ));
        help.set_xalign(0.0);
        main_box.append(&help);

        let search_entry = gtk::SearchEntry::new();
        search_entry
            .set_placeholder_text(Some("Search Lucide icons (e.g. rocket, folder, terminal)"));
        main_box.append(&search_entry);

        let settings_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let color_entry = gtk::Entry::new();
        color_entry.set_placeholder_text(Some("Color (#2563eb)"));
        color_entry.set_text("#2563eb");

        let stroke_spin = gtk::SpinButton::with_range(0.5, 4.0, 0.1);
        stroke_spin.set_value(2.0);
        stroke_spin.set_digits(1);

        let size_spin = gtk::SpinButton::with_range(12.0, 128.0, 1.0);
        size_spin.set_value(24.0);
        size_spin.set_digits(0);

        settings_row.append(&gtk::Label::new(Some("Color")));
        settings_row.append(&color_entry);
        settings_row.append(&gtk::Label::new(Some("Stroke")));
        settings_row.append(&stroke_spin);
        settings_row.append(&gtk::Label::new(Some("Size")));
        settings_row.append(&size_spin);
        main_box.append(&settings_row);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        main_box.append(&scrolled_window);

        let result_list = gtk::ListBox::new();
        result_list.set_selection_mode(gtk::SelectionMode::Single);
        result_list.add_css_class("boxed-list");
        scrolled_window.set_child(Some(&result_list));

        let status_label = gtk::Label::new(Some("Loading Lucide icons..."));
        status_label.set_xalign(0.0);
        main_box.append(&status_label);

        let selected_icon = Rc::new(RefCell::new(None));
        let lucide_service = LucideService::new().ok().map(Rc::new);

        let this = Self {
            dialog,
            search_entry,
            color_entry,
            stroke_spin,
            size_spin,
            result_list,
            status_label,
            selected_icon,
            lucide_service,
        };

        this.setup_events();
        this.refresh_results("");
        this
    }

    fn setup_events(&self) {
        let selected_icon = self.selected_icon.clone();
        self.result_list.connect_row_selected(move |_, row| {
            let selected = row.map(|r| r.widget_name().to_string());
            *selected_icon.borrow_mut() = selected;
        });

        let this = self.clone_handles();
        self.search_entry.connect_search_changed(move |entry| {
            this.refresh_results(&entry.text());
        });

        let this = self.clone_handles();
        self.color_entry.connect_changed(move |_| {
            this.refresh_results(&this.search_entry.text());
        });

        let this = self.clone_handles();
        self.stroke_spin.connect_value_changed(move |_| {
            this.refresh_results(&this.search_entry.text());
        });

        let this = self.clone_handles();
        self.size_spin.connect_value_changed(move |_| {
            this.refresh_results(&this.search_entry.text());
        });
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
            match service.search_icons(query, 48) {
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

        let mut preview_failures = 0usize;
        if let Some(service) = &self.lucide_service {
            preview_failures = populate_results(&self.result_list, service, &icons, &settings);
        } else {
            clear_list(&self.result_list);
        }

        self.status_label.set_text(&format!(
            "{} result(s){}",
            icons.len(),
            if preview_failures > 0 {
                format!(", {} preview(s) unavailable", preview_failures)
            } else {
                String::new()
            }
        ));

        if let Some(first) = self.result_list.first_child()
            && let Some(row) = first.downcast_ref::<gtk::ListBoxRow>()
        {
            self.result_list.select_row(Some(row));
        }
    }

    fn clone_handles(&self) -> Self {
        Self {
            dialog: self.dialog.clone(),
            search_entry: self.search_entry.clone(),
            color_entry: self.color_entry.clone(),
            stroke_spin: self.stroke_spin.clone(),
            size_spin: self.size_spin.clone(),
            result_list: self.result_list.clone(),
            status_label: self.status_label.clone(),
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

fn clear_list(result_list: &gtk::ListBox) {
    let mut children = Vec::new();
    let mut current = result_list.first_child();
    while let Some(widget) = current {
        current = widget.next_sibling();
        children.push(widget);
    }
    for child in children {
        result_list.remove(&child);
    }
}

fn populate_results(
    result_list: &gtk::ListBox,
    service: &LucideService,
    icons: &[String],
    settings: &LucideRenderSettings,
) -> usize {
    clear_list(result_list);

    let mut failures = 0usize;
    for icon_name in icons {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(icon_name);

        let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        match service.preview_icon_svg(icon_name, settings) {
            Ok(path) => {
                let image = gtk::Image::from_file(path);
                image.set_pixel_size(settings.size as i32);
                content.append(&image);
            }
            Err(_) => {
                failures += 1;
                let image = gtk::Image::from_icon_name("image-missing");
                image.set_pixel_size(16);
                content.append(&image);
            }
        }

        let label = gtk::Label::new(Some(icon_name));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        content.append(&label);

        row.set_child(Some(&content));
        result_list.append(&row);
    }

    failures
}
