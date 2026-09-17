use std::fs;

use crate::utils::image_uri;

const CONFIG_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml");

/// Grabs `player.background.image` from the config.toml file. Returns it
/// as a `file://` uri.
///
/// Returns `None` if the config toml file doesn't exist or there's a parsing
/// error.
pub fn player_background_uri() -> Option<String> {
    let contents = fs::read_to_string(CONFIG_TOML_PATH).ok()?;
    let root = contents.parse::<toml::Table>().ok()?;

    let image = root
        .get("player")?
        .as_table()?
        .get("background")?
        .as_table()?
        .get("image")?
        .as_str()?;

    image_uri(image)
}
