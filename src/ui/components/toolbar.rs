use gtk4::Button;
use gtk4::prelude::*;

// Header bar buttons for the sidebar page. Saving lives in the editor's bottom bar.
pub struct Toolbar {
    pub btn_new: Button,
    pub btn_open: Button,
    pub btn_refresh: Button,
}

pub fn build_toolbar() -> Toolbar {
    let btn_new = Button::from_icon_name("list-add-symbolic");
    btn_new.set_tooltip_text(Some("New .desktop"));

    let btn_open = Button::from_icon_name("document-open-symbolic");
    btn_open.set_tooltip_text(Some("Open"));

    let btn_refresh = Button::from_icon_name("view-refresh-symbolic");
    btn_refresh.set_tooltip_text(Some("Refresh"));

    Toolbar {
        btn_new,
        btn_open,
        btn_refresh,
    }
}
