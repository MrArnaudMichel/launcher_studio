use gtk4::Application;
use gtk4::PopoverMenuBar;
use gtk4::gio::Menu;

// Builds the application menu bar with File/View/Tools/Help/Credits.
// It wires no actions itself; it only defines the action names expected by the main window.
pub fn build_menu_bar(_app: &Application) -> PopoverMenuBar {
    let menu_model = Menu::new();

    // File menu
    let file_menu = Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Open"), Some("app.open"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Save As"), Some("app.save_as"));
    file_menu.append(Some("Refresh"), Some("app.refresh"));
    file_menu.append(Some("Quit"), Some("app.quit"));
    menu_model.append_submenu(Some("File"), &file_menu);

    // View menu
    let view_menu = Menu::new();
    view_menu.append(Some("Toggle Fullscreen"), Some("win.toggle_fullscreen"));
    menu_model.append_submenu(Some("View"), &view_menu);

    // Tools menu
    let tools_menu = Menu::new();
    tools_menu.append(
        Some("Open System Applications"),
        Some("app.open_system_dir"),
    );
    tools_menu.append(Some("Open User Applications"), Some("app.open_user_dir"));
    menu_model.append_submenu(Some("Tools"), &tools_menu);

    // Help menu
    let help_menu = Menu::new();
    help_menu.append(Some("About"), Some("app.about"));
    menu_model.append_submenu(Some("Help"), &help_menu);

    // Credits menu
    let credits_menu = Menu::new();
    credits_menu.append(Some("Show Credits"), Some("app.credits"));
    menu_model.append_submenu(Some("Credits"), &credits_menu);

    PopoverMenuBar::from_model(Some(&menu_model))
}
