pub mod menu_bar;
pub mod toolbar;
pub mod status_bar;
pub mod sidebar;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Label, Orientation};

pub fn labeled_entry_with(label: &str, entry: &Entry) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    let lbl = Label::new(Some(label));
    lbl.set_halign(gtk4::Align::End);
    lbl.set_xalign(1.0);
    lbl.set_width_chars(18);
    row.append(&lbl);
    row.append(entry);
    row
}

