//! Reads the config file and stores a reference to the values held in it.
//! The big function in this file used everywhere is the config() function.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::utils::{base_dir, image_uri, read_toml, resolve_path};

/// If no path found, uses the defaults found in this folder.
static CONFIG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
const DEFAULT_PLAYER_BACKGROUND: &str = "BF9560";
const DEFAULT_PLAYER_CLOCK_COLOR: &str = "1A1A1A";
const DEFAULT_PLAYER_CLOCK_REMAINING_COLOR: &str = "1A1A1A";
const DEFAULT_PLAYER_HINT_REMAINING_COLOR: &str = "1A1A1A";
const DEFAULT_PLAYER_HINT_AMOUNT_COLOR: &str = "1A1A1A";
const DEFAULT_PLAYER_HINT_COLOR: &str = "1A1A1A";
const DEFAULT_PLAYER_WIN_COLOR: &str = "89D154";
const DEFAULT_PLAYER_LOSE_COLOR: &str = "E32D2D";
const DEFAULT_PLAYER_WIN_TEXT: &str = "You win!";
const DEFAULT_PLAYER_LOSE_TEXT: &str = "You lose!";
const DEFAULT_PLAYER_VIDEO: &str = "./video.mp4";
const DEFAULT_MASTER_BACKGROUND: &str = "333333";
const DEFAULT_MASTER_NOISE: &str = "./audio.ogg";
// In seconds, 3600 seconds = 1 hour
const DEFAULT_MASTER_TIME: u32 = 3600;
const DEFAULT_FRAME_BORDER: &str = "D3D3D3";
const DEFAULT_FRAME_LABEL: &str = "D3D3D3";
const DEFAULT_EXPANDER: &str = "D3D3D3";
const DEFAULT_EXPANDER_HOVERED: &str = "FFFFFF";
const DEFAULT_BUTTON: &str = "1A1A1A";
const DEFAULT_BUTTON_BACKGROUND: &str = "D3D3D3";
const DEFAULT_BUTTON_LABEL: &str = "1A1A1A";
const DEFAULT_ENTRY_BACKGROUND: &str = "D3D3D3";
const DEFAULT_ENTRY_TEXT: &str = "1A1A1A";
const DEFAULT_HINTS_AMOUNT: u8 = 5;

/// Values pulled from config.toml
pub struct Config {
    pub player_background_image: Option<String>,
    pub player_background_color: String,
    pub player_clock_color: String,
    pub player_clock_remaining_color: String,
    pub player_hint_remaining_color: String,
    pub player_hint_amount_color: String,
    pub player_hint_color: String,
    pub player_win_color: String,
    pub player_lose_color: String,
    pub player_win_text: String,
    pub player_lose_text: String,
    pub player_video: Option<PathBuf>,
    pub master_background: String,
    pub master_noise: Option<PathBuf>,
    pub master_time: u32,
    pub master_log_path: Option<PathBuf>,
    pub master_room_name: String,
    pub frame_border: String,
    pub frame_label: String,
    pub expander_color: String,
    pub expander_hovered_color: String,
    pub button_color: String,
    pub button_background_color: String,
    pub button_label_color: String,
    pub entry_background_color: String,
    pub entry_text_color: String,
    pub hints_default_amount: u8,
}

/// Points the configuration at a file.
pub fn set_path(path: Option<PathBuf>) {
    let _ = CONFIG_PATH.set(path);
}

/// The loaded configuration. Read from disk on first use and cached.
pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    CONFIG.get_or_init(Config::load)
}

impl Config {
    /// Reads the configured file, falling back to the `DEFAULT_*` values key
    /// by key.
    fn load() -> Self {
        let path = CONFIG_PATH.get().and_then(Option::as_ref);
        let root = path.map(|path| read_toml(path)).unwrap_or_default();
        // Paths in the file are written relative to the file itself.
        let base = base_dir(path.map(PathBuf::as_path));

        Self {
            player_background_image: lookup(&root, "player.background.image")
                .and_then(toml::Value::as_str)
                .and_then(|value| image_uri(&base, value)),
            player_background_color: color(
                &root,
                "player.background.color",
                DEFAULT_PLAYER_BACKGROUND,
            ),
            player_clock_color: color(&root, "player.clock.color", DEFAULT_PLAYER_CLOCK_COLOR),
            player_clock_remaining_color: color(
                &root,
                "player.clock.remaining.color",
                DEFAULT_PLAYER_CLOCK_REMAINING_COLOR,
            ),
            player_hint_remaining_color: color(
                &root,
                "hint.remaining.color",
                DEFAULT_PLAYER_HINT_REMAINING_COLOR,
            ),
            player_hint_amount_color: color(
                &root,
                "hint.amount.color",
                DEFAULT_PLAYER_HINT_AMOUNT_COLOR,
            ),
            player_hint_color: color(&root, "hint.color", DEFAULT_PLAYER_HINT_COLOR),
            player_win_color: color(&root, "player.win.color", DEFAULT_PLAYER_WIN_COLOR),
            player_lose_color: color(&root, "player.lose.color", DEFAULT_PLAYER_LOSE_COLOR),
            player_win_text: text(&root, "player.win.text", DEFAULT_PLAYER_WIN_TEXT),
            player_lose_text: text(&root, "player.lose.text", DEFAULT_PLAYER_LOSE_TEXT),
            player_video: media(&root, &base, "player.video", DEFAULT_PLAYER_VIDEO),
            master_background: color(&root, "master.background", DEFAULT_MASTER_BACKGROUND),
            master_noise: media(&root, &base, "master.noise", DEFAULT_MASTER_NOISE),
            master_time: seconds(&root, "master.time", DEFAULT_MASTER_TIME),
            master_log_path: output(&root, &base, "master.log.path"),
            master_room_name: text(&root, "master.room.name", ""),
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
            hints_default_amount: amount(&root, "hint.default.amount", DEFAULT_HINTS_AMOUNT),
        }
    }
}

// Reads a line of text at the specified path. If the path isn't found,
// returns what is passed as default.
fn text(root: &toml::Table, path: &str, default: &str) -> String {
    lookup(root, path)
        .and_then(toml::Value::as_str)
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| default.to_string())
}

/// Reads a length of time in seconds, falling back to `default` when the key
/// isn't there or isn't a positive whole number.
fn seconds(root: &toml::Table, path: &str, default: u32) -> u32 {
    lookup(root, path)
        .and_then(toml::Value::as_integer)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|seconds| *seconds > 0)
        .unwrap_or(default)
}

/// Reads a count of something, falling back to `default` when the key isn't
/// there or isn't a whole number from 0 to 255.
fn amount(root: &toml::Table, path: &str, default: u8) -> u8 {
    lookup(root, path)
        .and_then(toml::Value::as_integer)
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(default)
}

/// Reads a color, falling back to `default` when it's invalid.
/// A leading `#` is accepted but not required.
fn color(root: &toml::Table, path: &str, default: &str) -> String {
    let value = text(root, path, default);
    let value = value.trim_start_matches('#');

    if is_hex(value) {
        value.to_string()
    } else {
        default.to_string()
    }
}

/// Reads a path to a media file, falling back to `default` when the key is
/// missing. `None` means there is no file there to play.
fn media(root: &toml::Table, base: &Path, path: &str, default: &str) -> Option<PathBuf> {
    let configured = lookup(root, path).and_then(toml::Value::as_str);

    resolve_path(base, configured.unwrap_or(default))
}

/// Reads a path to a file RoomWare writes to, so it may not exist yet. `None`
/// when the key is missing or empty.
fn output(root: &toml::Table, base: &Path, path: &str) -> Option<PathBuf> {
    lookup(root, path)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| base.join(value))
}

/// CSS accepts hex colors of 3, 4, 6 or 8 digits.
fn is_hex(value: &str) -> bool {
    matches!(value.len(), 3 | 4 | 6 | 8) && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Follows a dotted path such as `player.background.color` down the nested
/// tables that toml builds for both `[a.b]` headers and `a.b = x` keys.
fn lookup<'a>(root: &'a toml::Table, path: &str) -> Option<&'a toml::Value> {
    let mut keys = path.split('.');
    let mut value = root.get(keys.next()?)?;

    for key in keys {
        value = value.as_table()?.get(key)?;
    }

    Some(value)
}
