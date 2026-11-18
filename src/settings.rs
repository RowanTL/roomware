//! The web server's password and on/off switch, kept in `server.toml`.

use std::path::{Path, PathBuf};
use std::{fs, io};

use gtk::prelude::{ButtonExt, EditableExt, GridExt, GtkWindowExt, WidgetExt};
use relm4::gtk;

use crate::rooms;

const FILE: &str = "server.toml";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub password: String,
    pub is_active: bool,
}

pub fn path() -> PathBuf {
    rooms::installed_dir().with_file_name(FILE)
}

impl Settings {
    /// Writes the defaults out when the file has no settings in it yet.
    pub fn load(path: &Path) -> Self {
        let root = fs::read_to_string(path)
            .ok()
            .and_then(|contents| contents.parse::<toml::Table>().ok());
        let server = root
            .as_ref()
            .and_then(|root| root.get("server"))
            .and_then(toml::Value::as_table);

        let Some(server) = server else {
            let settings = Self::default();
            if let Err(error) = settings.save(path) {
                eprintln!("roomware: could not write {}: {error}", path.display());
            }
            return settings;
        };

        Self {
            password: server
                .get("password")
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string(),
            is_active: server
                .get("is_active")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut server = toml::Table::new();
        server.insert("password".to_string(), self.password.clone().into());
        server.insert("is_active".to_string(), self.is_active.into());

        let mut root = toml::Table::new();
        root.insert("server".to_string(), server.into());

        fs::write(path, root.to_string())
    }

    /// A blank password counts as off, since it would let anyone in.
    pub fn should_serve(&self) -> bool {
        self.is_active && !self.password.is_empty()
    }
}

/// Opens the settings window; `server_status` is shown as is.
pub fn show_dialog(
    parent: Option<&gtk::Window>,
    settings: &Settings,
    server_status: &str,
    on_save: impl Fn(Settings) + 'static,
    on_check_for_updates: impl Fn() + 'static,
    on_update_hints: impl Fn() + 'static,
) {
    let password = gtk::Entry::builder()
        .text(settings.password.as_str())
        .hexpand(true)
        .width_chars(30)
        .build();
    let active = gtk::Switch::builder()
        .active(settings.should_serve())
        .sensitive(!settings.password.is_empty())
        .halign(gtk::Align::Start)
        .build();
    let save = gtk::Button::with_label("Save");
    let check = gtk::Button::with_label("Check for Updates");
    let hints = gtk::Button::with_label("Update Hints");
    let close = gtk::Button::with_label("Close");

    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    let label = |text: &str| gtk::Label::builder().label(text).xalign(0.0).build();
    grid.attach(&label("Web page password"), 0, 0, 1, 1);
    grid.attach(&password, 1, 0, 1, 1);
    grid.attach(&label("Web server"), 0, 1, 1, 1);
    grid.attach(&active, 1, 1, 1, 1);
    grid.attach(&label(server_status), 0, 2, 2, 1);
    grid.attach(&save, 0, 3, 2, 1);
    grid.attach(&check, 0, 4, 2, 1);
    grid.attach(&hints, 0, 5, 2, 1);
    grid.attach(&close, 0, 6, 2, 1);

    let dialog = gtk::Window::builder()
        .title("Settings")
        .modal(true)
        .resizable(false)
        .child(&grid)
        .build();
    dialog.set_transient_for(parent);

    let window = dialog.clone();
    close.connect_clicked(move |_| window.close());

    let window = dialog.clone();
    check.connect_clicked(move |_| {
        on_check_for_updates();
        window.close();
    });

    let window = dialog.clone();
    hints.connect_clicked(move |_| {
        on_update_hints();
        window.close();
    });

    // The server can't be on without a password.
    let switch = active.clone();
    password.connect_changed(move |password| {
        let blank = password.text().trim().is_empty();
        switch.set_sensitive(!blank);
        if blank {
            switch.set_active(false);
        }
    });

    let window = dialog.clone();
    save.connect_clicked(move |_| {
        on_save(Settings {
            password: password.text().trim().to_string(),
            is_active: active.is_active(),
        });
        window.close();
    });

    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "roomware-server-{name}-{}.toml",
            std::process::id()
        ))
    }

    #[test]
    fn a_missing_file_is_generated_with_the_server_off() {
        let path = temp_path("missing");
        let _ = fs::remove_file(&path);

        assert_eq!(Settings::load(&path), Settings::default());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "[server]\npassword = \"\"\nis_active = false\n"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn saved_settings_are_loaded_back() {
        let path = temp_path("round-trip");
        let settings = Settings {
            password: "pässwörd \"quoted\"".to_string(),
            is_active: true,
        };

        settings.save(&path).unwrap();

        assert_eq!(Settings::load(&path), settings);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn whitespace_around_the_password_is_trimmed() {
        let path = temp_path("trimmed");
        fs::write(&path, "[server]\npassword=\"  secret \"\nis_active=true\n").unwrap();

        assert_eq!(Settings::load(&path).password, "secret");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn an_active_server_with_a_blank_password_is_not_served() {
        let settings = Settings {
            password: String::new(),
            is_active: true,
        };

        assert!(!settings.should_serve());
    }
}
