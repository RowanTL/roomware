//! The web server's password and on/off switch, kept in `server.toml`.

use std::path::{Path, PathBuf};
use std::{fs, io};

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
