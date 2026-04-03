use gtk4 as gtk;
use gtk::prelude::*;
use std::rc::Rc;
use crate::services::icon_service::IconService;

pub struct IconPickerDialog {
    dialog: gtk::Dialog,
    search_entry: gtk::SearchEntry,
    icon_list: gtk::FlowBox,
    selected_icon: Rc<std::cell::RefCell<Option<String>>>,
    icon_service: IconService,
}

impl IconPickerDialog {
    pub fn new(parent: Option<&impl IsA<gtk::Window>>) -> Self {
        let dialog = gtk::Dialog::builder()
            .title("Select Icon")
            .modal(true)
            .default_width(500)
            .default_height(400)
            .build();
        
        if let Some(p) = parent {
            dialog.set_transient_for(Some(p));
        }

        dialog.add_buttons(&[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Select", gtk::ResponseType::Accept),
        ]);

        let content_area = dialog.content_area();
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        main_box.set_margin_top(10);
        main_box.set_margin_bottom(10);
        main_box.set_margin_start(10);
        main_box.set_margin_end(10);
        content_area.append(&main_box);

        let search_entry = gtk::SearchEntry::new();
        main_box.append(&search_entry);

        let scrolled_window = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();
        main_box.append(&scrolled_window);

        let icon_list = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .max_children_per_line(10)
            .min_children_per_line(5)
            .build();
        scrolled_window.set_child(Some(&icon_list));

        let selected_icon = Rc::new(std::cell::RefCell::new(None));
        let icon_service = IconService::new();

        let this = Self {
            dialog,
            search_entry,
            icon_list,
            selected_icon,
            icon_service,
        };

        this.setup_events();
        this.populate_icons("");

        this
    }

    fn setup_events(&self) {
        let icon_list = self.icon_list.clone();
        let selected_icon = self.selected_icon.clone();
        
        icon_list.connect_child_activated(move |_, child| {
            if let Some(btn) = child.child().and_downcast::<gtk::Button>() {
                if let Some(icon_name) = btn.icon_name() {
                    *selected_icon.borrow_mut() = Some(icon_name.to_string());
                }
            }
        });
    }

    fn populate_icons(&self, query: &str) {
        // Clear existing icons
        while let Some(child) = self.icon_list.first_child() {
            self.icon_list.remove(&child);
        }

        let icons = if query.is_empty() {
            // Limit to some common icons for performance if no query? 
            // Or just show all. icon_names() can be thousands.
            self.icon_service.list_icons().into_iter().take(200).collect::<Vec<_>>()
        } else {
            self.icon_service.search_icons(query).into_iter().take(200).collect::<Vec<_>>()
        };

        for name in icons {
            let btn = gtk::Button::from_icon_name(&name);
            btn.set_tooltip_text(Some(&name));
            btn.set_has_frame(false);
            let _ = self.icon_list.insert(&btn, -1);
        }
    }

    pub fn run<F: FnOnce(Option<String>) + 'static>(self, callback: F) {
        let selected_icon = self.selected_icon.clone();
        let dialog = self.dialog.clone();
        
        // Manual search event handling since I can't easily capture 'self' in closure
        let icon_list = self.icon_list.clone();
        let icon_service = IconService::new(); // Ideally shared but for now...
        self.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            // Clear
            while let Some(child) = icon_list.first_child() {
                icon_list.remove(&child);
            }
            let icons = if query.is_empty() {
                icon_service.list_icons().into_iter().take(200).collect::<Vec<_>>()
            } else {
                icon_service.search_icons(&query).into_iter().take(200).collect::<Vec<_>>()
            };
            for name in icons {
                let btn = gtk::Button::from_icon_name(&name);
                btn.set_tooltip_text(Some(&name));
                btn.set_has_frame(false);
                let _ = icon_list.insert(&btn, -1);
            }
        });

        let callback_cell = std::rc::Rc::new(std::cell::RefCell::new(Some(callback)));

        dialog.connect_response(move |d, resp| {
            if let Some(cb) = callback_cell.borrow_mut().take() {
                if resp == gtk::ResponseType::Accept {
                    cb(selected_icon.borrow().clone());
                } else {
                    cb(None);
                }
            }
            d.close();
        });
        dialog.show();
    }
}
