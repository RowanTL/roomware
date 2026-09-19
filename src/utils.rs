use std::path::{Path, PathBuf};
use std::{env, fs};

use relm4::gtk;

/// Load `text` as an image if it is one.
pub fn load_image(base: &Path, text: &str) -> Option<gtk::gdk::Texture> {
    let path = resolve_path(base, text)?;
    gtk::gdk::Texture::from_filename(&path).ok()
}

/// Resolve `text` to a `file://` URI for use in a CSS `url(...)` if it points
/// at a readable image.
pub fn image_uri(base: &Path, text: &str) -> Option<String> {
    let path = resolve_path(base, text)?;
    gtk::gdk::Texture::from_filename(&path).ok()?;

    gtk::glib::filename_to_uri(&path, None).ok().map(Into::into)
}

/// Relative paths are resolved against `base`, the directory of the toml file
/// they were read from, so that entries like `./picture.png` mean the picture
/// sitting next to that file no matter where the binary is run from.
pub fn resolve_path(base: &Path, text: &str) -> Option<PathBuf> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    let path = Path::new(text);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };

    // Handles `.` and `..` sometimes found in paths
    let path = path.canonicalize().ok()?;

    path.is_file().then_some(path)
}

/// The directory the entries of a toml file are relative to: the one holding
/// it, or the working directory when no file was given at all.
pub fn base_dir(path: Option<&Path>) -> PathBuf {
    path.and_then(Path::parent)
        // `Path::parent` of a bare "hints.toml" is empty, which is the
        // working directory rather than the root of the filesystem.
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Reads and parses a toml file. One that was asked for but can't be read or
/// parsed is worth saying out loud; the caller carries on with an empty table
/// either way.
pub fn read_toml(path: &Path) -> toml::Table {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("roomware: could not read {}: {error}", path.display());
            return toml::Table::new();
        }
    };

    match contents.parse::<toml::Table>() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("roomware: could not parse {}: {error}", path.display());
            toml::Table::new()
        }
    }
}

/// Applies CSS styling to whatever object this function is called from.
pub fn apply_css_style(css_string: &str) {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let provider = gtk::CssProvider::new();
    provider.load_from_string(css_string);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
