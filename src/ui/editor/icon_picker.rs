use crate::services::icon_service::IconService;
use gtk::prelude::*;
use gtk4 as gtk;
use std::rc::Rc;

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

        let selected_icon_for_activate = selected_icon.clone();
        icon_list.connect_child_activated(move |_, child| {
            if let Some(btn) = child.child().and_downcast::<gtk::Button>()
                && let Some(icon_name) = btn.icon_name()
            {
                *selected_icon_for_activate.borrow_mut() = Some(icon_name.to_string());
            }
        });

        icon_list.connect_selected_children_changed(move |flow| {
            let selected = flow
                .selected_children()
                .into_iter()
                .next()
                .and_then(|child| {
                    child
                        .child()
                        .and_downcast::<gtk::Button>()
                        .and_then(|btn| btn.icon_name())
                        .map(|name| name.to_string())
                });
            *selected_icon.borrow_mut() = selected;
        });
    }

    fn populate_icons(&self, query: &str) {
        let candidates = if query.trim().is_empty() {
            self.icon_service.list_icons()
        } else {
            self.icon_service.search_icons(query)
        };
        let icons = pick_icons_for_display(&candidates);
        populate_icon_list(&self.icon_list, &self.selected_icon, icons);
    }

    pub fn run<F: FnOnce(Option<String>) + 'static>(self, callback: F) {
        let selected_icon = self.selected_icon.clone();
        let dialog = self.dialog.clone();

        let icon_list = self.icon_list.clone();
        let all_icons = Rc::new(self.icon_service.list_icons());
        let all_icons_for_response = all_icons.clone();
        let selected_for_search = selected_icon.clone();
        self.search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            let icons = pick_icons_for_query(&all_icons, &query);
            populate_icon_list(&icon_list, &selected_for_search, icons);
        });

        let callback_cell = std::rc::Rc::new(std::cell::RefCell::new(Some(callback)));

        dialog.connect_response(move |d, resp| {
            if let Some(cb) = callback_cell.borrow_mut().take() {
                if resp == gtk::ResponseType::Accept {
                    cb(normalize_icon_name_for_desktop(
                        selected_icon.borrow().clone(),
                        &all_icons_for_response,
                    ));
                } else {
                    cb(None);
                }
            }
            d.close();
        });
        dialog.show();
    }
}

fn pick_icons_for_query(all_icons: &[String], query: &str) -> Vec<String> {
    let q = query.trim().to_lowercase();
    let filtered: Vec<String> = all_icons
        .iter()
        .filter(|name| q.is_empty() || name.to_lowercase().contains(&q))
        .cloned()
        .collect();
    pick_icons_for_display(&filtered)
}

fn pick_icons_for_display(candidates: &[String]) -> Vec<String> {
    let mut regular = Vec::new();
    let mut symbolic = Vec::new();

    for name in candidates {
        if name.starts_with("adw-") {
            continue;
        }
        if name.ends_with("-symbolic") {
            symbolic.push(name.clone());
        } else {
            regular.push(name.clone());
        }
    }

    regular.extend(symbolic);
    regular.truncate(200);
    regular
}

fn normalize_icon_name_for_desktop(
    selected: Option<String>,
    all_icons: &[String],
) -> Option<String> {
    let name = selected?;
    if let Some(base) = name.strip_suffix("-symbolic")
        && all_icons.iter().any(|n| n == base)
    {
        return Some(base.to_string());
    }
    if name.starts_with("adw-") {
        return None;
    }
    Some(name)
}

fn populate_icon_list(
    icon_list: &gtk::FlowBox,
    selected_icon: &Rc<std::cell::RefCell<Option<String>>>,
    icons: Vec<String>,
) {
    while let Some(child) = icon_list.first_child() {
        icon_list.remove(&child);
    }

    for name in icons {
        let btn = gtk::Button::from_icon_name(&name);
        btn.set_tooltip_text(Some(&name));
        btn.set_has_frame(false);
        {
            let selected_icon = selected_icon.clone();
            let selected_name = name.clone();
            btn.connect_clicked(move |_| {
                *selected_icon.borrow_mut() = Some(selected_name.clone());
            });
        }
        icon_list.insert(&btn, -1);
    }
}
