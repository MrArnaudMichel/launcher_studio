use gtk4::{ResponseType, Window};
use adw::{AboutDialog, prelude::*};

pub fn show_error<W: IsA<Window>>(parent: &W, msg: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Error")
        .text("Operation failed")
        .secondary_text(msg)
        .build();
    dialog.add_button("Close", ResponseType::Close);
    dialog.connect_response(|d, _| d.close());
    dialog.show();
}

pub fn show_about<W: IsA<gtk4::Widget>>(parent: &W) {
    let about = AboutDialog::new();
    about.set_application_name("Desktop Entry Manager");
    about.set_developer_name("Arnaud Michel");
    about.set_version(env!("CARGO_PKG_VERSION"));
    about.set_website("https://github.com/MrArnaudMichel/launcher_studio");
    about.set_issue_url("https://github.com/MrArnaudMichel/launcherstudio/issues");
    about.present(Some(parent));
}

pub fn show_credits<W: IsA<Window>>(parent: &W) {
    let text = "Desktop Entry Manager\n\nCredits:\n- Author: Arnaud Michel\n- UI: GTK4 + Libadwaita";
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Credits")
        .text("Thanks for using Desktop Entry Manager")
        .secondary_text(text)
        .build();
    dialog.add_button("Close", ResponseType::Close);
    dialog.connect_response(|d, _| d.close());
    dialog.show();
}

pub fn confirm_delete<W: IsA<Window>, F>(parent: &W, path: &std::path::Path, on_confirm: F)
where
    F: Fn() + 'static,
{
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Confirm deletion")
        .text("Delete selected .desktop file?")
        .secondary_text(&format!("This will permanently remove:\n{}", path.display()))
        .build();
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("Delete", ResponseType::Accept);
    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept {
            on_confirm();
        }
        d.close();
    });
    dialog.show();
}

pub fn show_save_success<W: IsA<Window>>(parent: &W, path: std::path::PathBuf, is_update: bool) {
    let (title, text) = if is_update {
        ("Saved", ".desktop file updated")
    } else {
        ("Saved", ".desktop file created")
    };
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .text(text)
        .secondary_text(&format!("{} {}", if is_update { "Updated" } else { "Saved to" }, path.display()))
        .build();
    dialog.add_button("Open Folder", ResponseType::Accept);
    dialog.add_button("Close", ResponseType::Close);
    dialog.connect_response(move |d, resp| {
        if resp == ResponseType::Accept {
            #[cfg(target_os = "linux")]
            if let Some(parent) = path.parent() {
                let _ = open::that(parent);
            }
        }
        d.close();
    });
    dialog.show();
}

pub fn show_preview<W: IsA<Window>>(parent: &W, content: &str) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Preview .desktop")
        .text("This is the generated .desktop content:")
        .secondary_text(content)
        .build();
    dialog.add_button("Close", ResponseType::Close);
    dialog.connect_response(|d, _| d.close());
    dialog.show();
}
