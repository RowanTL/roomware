use std::fs;
use std::sync::OnceLock;

use crate::utils::image_uri;

const CONFIG_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");
const DEFAULT_PLAYER_BACKGROUND: &str = "BF9560";
const DEFAULT_PLAYER_LABEL: &str = "1A1A1A";
const DEFAULT_MASTER_BACKGROUND: &str = "333333";
const DEFAULT_FRAME_BORDER: &str = "D3D3D3";
const DEFAULT_FRAME_LABEL: &str = "D3D3D3";
const DEFAULT_EXPANDER: &str = "D3D3D3";
const DEFAULT_EXPANDER_HOVERED: &str = "FFFFFF";
const DEFAULT_BUTTON: &str = "1A1A1A";
const DEFAULT_BUTTON_BACKGROUND: &str = "D3D3D3";
const DEFAULT_BUTTON_LABEL: &str = "1A1A1A";
const DEFAULT_ENTRY_BACKGROUND: &str = "D3D3D3";
const DEFAULT_ENTRY_TEXT: &str = "1A1A1A";

/// Values pulled from config.toml
pub struct Theme {
    pub player_background_image: Option<String>,
    pub player_background_color: String,
    pub player_label_color: String,
    pub master_background: String,
    pub frame_border: String,
    pub frame_label: String,
    pub expander_color: String,
    pub expander_hovered_color: String,
    pub button_color: String,
    pub button_background_color: String,
    pub button_label_color: String,
    pub entry_background_color: String,
    pub entry_text_color: String,
}

/// The configured theme. Read from disk on first use and cache
pub fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();

    THEME.get_or_init(Theme::load)
}

impl Theme {
    /// Reads config.toml, falling back to the `DEFAULT_*` colors key by key.
    fn load() -> Self {
        // TODO: Make this accept a path later :/
        let root = fs::read_to_string(CONFIG_TOML_PATH)
            .ok()
            .and_then(|contents| contents.parse::<toml::Table>().ok())
            .unwrap_or_default();

        Self {
            player_background_image: lookup(&root, "player.background.image")
                .as_deref()
                .and_then(image_uri),
            player_background_color: color(
                &root,
                "player.background.color",
                DEFAULT_PLAYER_BACKGROUND,
            ),
            player_label_color: color(&root, "player.label.color", DEFAULT_PLAYER_LABEL),
            master_background: color(&root, "master.background", DEFAULT_MASTER_BACKGROUND),
            frame_border: color(&root, "frame.border", DEFAULT_FRAME_BORDER),
            frame_label: color(&root, "frame.label", DEFAULT_FRAME_LABEL),
            expander_color: color(&root, "expander.color", DEFAULT_EXPANDER),
            expander_hovered_color: color(
                &root,
                "expander.hovered.color",
                DEFAULT_EXPANDER_HOVERED,
            ),
            button_color: color(&root, "button.color", DEFAULT_BUTTON),
            button_background_color: color(
                &root,
                "button.background.color",
                DEFAULT_BUTTON_BACKGROUND,
            ),
            button_label_color: color(&root, "button.label.color", DEFAULT_BUTTON_LABEL),
            entry_background_color: color(
                &root,
                "entry.background.color",
                DEFAULT_ENTRY_BACKGROUND,
            ),
            entry_text_color: color(&root, "entry.text.color", DEFAULT_ENTRY_TEXT),
        }
    }
}

/// Reads a color, falling back to `default` when it's invalid.
/// A leading `#` is accepted but not required.
fn color(root: &toml::Table, path: &str, default: &str) -> String {
    lookup(root, path)
        .map(|value| value.trim().trim_start_matches('#').to_string())
        .filter(|value| is_hex(value))
        .unwrap_or_else(|| default.to_string())
}

/// CSS accepts hex colors of 3, 4, 6 or 8 digits.
fn is_hex(value: &str) -> bool {
    matches!(value.len(), 3 | 4 | 6 | 8) && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Follows a dotted path such as `player.background.color` down the nested
/// tables that toml builds for both `[a.b]` headers and `a.b = x` keys.
fn lookup(root: &toml::Table, path: &str) -> Option<String> {
    let mut keys = path.split('.');
    let mut value = root.get(keys.next()?)?;

    for key in keys {
        value = value.as_table()?.get(key)?;
    }

    Some(value.as_str()?.to_string())
}
