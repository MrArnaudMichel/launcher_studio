use adw::prelude::*;
use adw::Application;
use crate::ui;

pub fn run() {
    let _ = adw::init();

    let app = Application::builder()
        .application_id("fr.arnaudmichel.launcherstudio")
        .build();

    app.connect_activate(|app| {
        ui::windows::main_window::show_main_window(app);
    });

    app.run();
}
