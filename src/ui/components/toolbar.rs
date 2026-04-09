use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Orientation};

pub struct Toolbar {
    pub container: GtkBox,
    pub btn_new: Button,
    pub btn_open: Button,
    pub btn_save: Button,
    pub btn_refresh: Button,
}

pub fn build_toolbar() -> Toolbar {
    let container = GtkBox::new(Orientation::Horizontal, 6);

    let btn_new = Button::new();
    let img_new = Image::from_icon_name("list-add-symbolic");
    img_new.set_pixel_size(18);
    btn_new.set_child(Some(&img_new));
    btn_new.set_tooltip_text(Some("New .desktop"));

    let btn_open = Button::new();
    let img_open = Image::from_icon_name("document-open-symbolic");
    img_open.set_pixel_size(18);
    btn_open.set_child(Some(&img_open));
    btn_open.set_tooltip_text(Some("Open"));

    let btn_save = Button::new();
    let img_save = Image::from_icon_name("document-save-symbolic");
    img_save.set_pixel_size(18);
    btn_save.set_child(Some(&img_save));
    btn_save.set_tooltip_text(Some("Save"));

    let btn_refresh = Button::new();
    let img_refresh = Image::from_icon_name("view-refresh-symbolic");
    img_refresh.set_pixel_size(18);
    btn_refresh.set_child(Some(&img_refresh));
    btn_refresh.set_tooltip_text(Some("Refresh"));

    container.append(&btn_new);
    container.append(&btn_open);
    container.append(&btn_save);
    container.append(&btn_refresh);

    Toolbar {
        container,
        btn_new,
        btn_open,
        btn_save,
        btn_refresh,
    }
}
