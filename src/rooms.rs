//! Downloads rooms from the public rooms repo and keeps them up to date.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::Value;

const LISTING_URL: &str = "https://api.github.com/repos/RowanTL/rooms/contents";
const RAW_URL: &str = "https://raw.githubusercontent.com/RowanTL/rooms/main";
const QUICK_TIMEOUT: Duration = Duration::from_secs(10);

pub const CONFIG_FILE: &str = "config.toml";
pub const HINTS_FILE: &str = "hints.toml";

/// Only set when the room is downloaded rather than read straight from disk.
static ROOM: OnceLock<(String, PathBuf)> = OnceLock::new();

type Error = Box<dyn std::error::Error>;

/// Where an installed RoomWare keeps its rooms, beside its `bin` directory.
pub fn installed_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.parent()?.join("rooms")))
        .unwrap_or_else(|| PathBuf::from("rooms"))
}

/// Brings `dir` up to date with the room called `name`, downloading only the
/// files that are missing or changed unless `force` is set. The hints are
/// left to [`update_hints`]. Room variable initialized here.
pub fn sync(name: &str, dir: &Path, force: bool) {
    let _ = ROOM.set((name.to_string(), dir.to_path_buf()));

    if let Err(error) = try_sync(name, dir, force) {
        eprintln!("roomware: could not update room {name}: {error}");
    }
}

/// Fetches the latest hints for the downloaded room, if there is one.
pub fn update_hints() {
    let Some((name, dir)) = ROOM.get() else {
        return;
    };

    let url = format!("{RAW_URL}/{name}/{HINTS_FILE}");
    if let Err(error) = download(&url, &dir.join(HINTS_FILE), Some(QUICK_TIMEOUT)) {
        eprintln!("roomware: could not update hints for {name}: {error}");
    }
}

fn try_sync(name: &str, dir: &Path, force: bool) -> Result<(), Error> {
    let listing: Vec<Value> = client()
        .get(format!("{LISTING_URL}/{name}"))
        .timeout(QUICK_TIMEOUT)
        .send()?
        .error_for_status()?
        .json()?;

    fs::create_dir_all(dir)?;

    for entry in &listing {
        let (Some("file"), Some(file), Some(sha), Some(url)) = (
            entry["type"].as_str(),
            entry["name"].as_str(),
            entry["sha"].as_str(),
            entry["download_url"].as_str(),
        ) else {
            continue;
        };

        let path = dir.join(file);
        // skip downloading if local hints file or sha's don't match
        if file == HINTS_FILE || (!force && blob_sha(&path).as_deref() == Some(sha)) {
            continue;
        }

        download(url, &path, None)?;
    }

    Ok(())
}

/// Writes to a temporary file first so a failed download can't leave half a
/// file behind.
fn download(url: &str, path: &Path, timeout: Option<Duration>) -> Result<(), Error> {
    let mut request = client().get(url);
    if let Some(timeout) = timeout {
        request = request.timeout(timeout);
    }
    let mut response = request.send()?.error_for_status()?;

    let partial = path.with_extension("part");
    response.copy_to(&mut File::create(&partial)?)?;
    fs::rename(&partial, path)?;

    Ok(())
}

/// The hash git gives a file, which is what the listing reports for each one.
fn blob_sha(path: &Path) -> Option<String> {
    let contents = fs::read(path).ok()?;

    let mut sha = sha1_smol::Sha1::new();
    sha.update(format!("blob {}\0", contents.len()).as_bytes());
    sha.update(&contents);

    Some(sha.digest().to_string())
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();

    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("roomware")
            .connect_timeout(QUICK_TIMEOUT)
            // Room videos can take a while on a slow connection.
            .timeout(None)
            .build()
            .expect("the HTTP client's settings are fixed")
    })
}
