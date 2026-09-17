use std::path::{Path, PathBuf};

use relm4::gtk;

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Load `text` as an image if it is one.
pub fn load_image(text: &str) -> Option<gtk::gdk::Texture> {
    let path = resolve_path(text)?;
    gtk::gdk::Texture::from_filename(&path).ok()
}

/// Resolve `text` to a `file://` URI for use in a CSS `url(...)` if it points
/// at a readable image.
pub fn image_uri(text: &str) -> Option<String> {
    let path = resolve_path(text)?;
    gtk::gdk::Texture::from_filename(&path).ok()?;

    gtk::glib::filename_to_uri(&path, None).ok().map(Into::into)
}

/// Relative paths are resolved against the project root so that `hints.toml`
/// entries like `./picture.png` work no matter where the binary is run from.
fn resolve_path(text: &str) -> Option<PathBuf> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    let path = Path::new(text);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(PROJECT_ROOT).join(path)
    };

    // Handles `.` and `..` sometimes found in paths
    let path = path.canonicalize().ok()?;

    path.is_file().then_some(path)
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
