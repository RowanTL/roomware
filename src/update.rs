//! Finds a newer release on GitHub and hands it to the Windows installer.

use std::path::Path;

use serde_json::Value;

use crate::rooms;

const LATEST_URL: &str = "https://api.github.com/repos/RowanTL/roomware/releases/latest";
const INSTALLER: &str = "roomware-setup.exe";

#[derive(Debug, PartialEq)]
pub struct Update {
    pub version: String,
    installer_url: String,
}

/// The latest release, if it is newer than this build.
pub fn check() -> Result<Option<Update>, String> {
    let release: Value = rooms::client()
        .get(LATEST_URL)
        .timeout(rooms::QUICK_TIMEOUT)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.json())
        .map_err(|error| error.to_string())?;

    newer(&release, env!("CARGO_PKG_VERSION"))
}

/// Downloads the installer and starts it. RoomWare has to quit afterwards so its files can be replaced.
pub fn install(update: &Update) -> Result<(), String> {
    let installer = std::env::temp_dir().join(INSTALLER);
    rooms::download(&update.installer_url, &installer, None).map_err(|error| error.to_string())?;

    launch(&installer)
}

fn newer(release: &Value, current: &str) -> Result<Option<Update>, String> {
    let tag = release["tag_name"]
        .as_str()
        .ok_or("the latest release has no tag")?;
    let (Some(latest), Some(running)) = (parse(tag), parse(current)) else {
        return Err(format!("can't compare version {tag} with {current}"));
    };
    if latest <= running {
        return Ok(None);
    }

    let installer_url = release["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|asset| asset["name"] == INSTALLER)
        .and_then(|asset| asset["browser_download_url"].as_str())
        .ok_or_else(|| format!("release {tag} has no {INSTALLER}"))?;

    Ok(Some(Update {
        version: tag.trim_start_matches('v').to_string(),
        installer_url: installer_url.to_string(),
    }))
}

fn parse(version: &str) -> Option<Vec<u64>> {
    version
        .trim_start_matches('v')
        .split('.')
        .map(|part| part.parse().ok())
        .collect()
}

/// Elevated, since the installer writes to Program Files. Windows asks the user first.
#[cfg(windows)]
fn launch(installer: &Path) -> Result<(), String> {
    use std::ffi::{OsStr, c_void};
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn ShellExecuteW(
            window: *mut c_void,
            verb: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show: i32,
        ) -> isize;
    }

    let wide = |text: &OsStr| -> Vec<u16> { text.encode_wide().chain([0]).collect() };
    let verb = wide(OsStr::new("runas"));
    let file = wide(installer.as_os_str());
    // See nix/installer.nsi for both.
    let parameters = wide(OsStr::new("/S /RELAUNCH"));

    // SAFETY: every string is NUL-terminated and outlives the call.
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            ptr::null(),
            1,
        )
    };

    // ShellExecute reports a failure, such as a declined prompt, as 32 or less.
    if result > 32 {
        Ok(())
    } else {
        Err(format!(
            "Windows did not start the installer (code {result})"
        ))
    }
}

#[cfg(not(windows))]
fn launch(_installer: &Path) -> Result<(), String> {
    Err("updates can only be installed on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn release(tag: &str) -> Value {
        json!({
            "tag_name": tag,
            "assets": [
                { "name": "roomware.9.9.9.nupkg", "browser_download_url": "https://example.com/nupkg" },
                { "name": INSTALLER, "browser_download_url": "https://example.com/setup" },
            ],
        })
    }

    #[test]
    fn a_newer_release_is_an_update_with_its_installer() {
        let update = newer(&release("v4.2.0"), "4.1.0").unwrap().unwrap();

        assert_eq!(update.version, "4.2.0");
        assert_eq!(update.installer_url, "https://example.com/setup");
    }

    #[test]
    fn the_same_or_an_older_release_is_no_update() {
        assert_eq!(newer(&release("v4.1.0"), "4.1.0"), Ok(None));
        assert_eq!(newer(&release("v4.0.9"), "4.1.0"), Ok(None));
    }

    #[test]
    fn versions_compare_by_number_rather_than_by_text() {
        assert!(newer(&release("v4.10.0"), "4.9.0").unwrap().is_some());
    }

    #[test]
    fn a_newer_release_without_an_installer_is_an_error() {
        let release = json!({ "tag_name": "v9.0.0", "assets": [] });

        assert!(newer(&release, "4.1.0").is_err());
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_an_error() {
        assert!(newer(&release("nightly"), "4.1.0").is_err());
    }
}
