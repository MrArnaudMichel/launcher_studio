use gtk4::MenuButton;
use gtk4::gio::Menu;
use gtk4::prelude::*;

// Builds the primary (hamburger) menu button for the header bar.
// It wires no actions itself; it only defines the action names expected by the main window.
pub fn build_primary_menu_button() -> MenuButton {
    let menu_model = Menu::new();

    let file_section = Menu::new();
    file_section.append(Some("New"), Some("app.new"));
    file_section.append(Some("Open..."), Some("app.open"));
    file_section.append(Some("Import System Launcher..."), Some("app.import_system"));
    file_section.append(Some("Save"), Some("app.save"));
    file_section.append(Some("Save As..."), Some("app.save_as"));
    file_section.append(Some("Refresh"), Some("app.refresh"));
    menu_model.append_section(None, &file_section);

    let view_section = Menu::new();
    view_section.append(Some("Find Launcher..."), Some("win.focus_search"));
    view_section.append(Some("Toggle Fullscreen"), Some("win.toggle_fullscreen"));
    menu_model.append_section(None, &view_section);

    let tools_section = Menu::new();
    tools_section.append(
        Some("Open System Applications"),
        Some("app.open_system_dir"),
    );
    tools_section.append(Some("Open User Applications"), Some("app.open_user_dir"));
    menu_model.append_section(None, &tools_section);

    let help_section = Menu::new();
    help_section.append(Some("Credits"), Some("app.credits"));
    help_section.append(Some("About Launcher Studio"), Some("app.about"));
    help_section.append(Some("Quit"), Some("app.quit"));
    menu_model.append_section(None, &help_section);

    let button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Main Menu")
        .menu_model(&menu_model)
        .build();
    // Opens with F10; set by name since the gtk4 crate is built without the v4_4 feature.
    button.set_property("primary", true);
    button
}
