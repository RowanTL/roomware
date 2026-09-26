// Keeps Windows from opening a console window beside the game master's.
#![windows_subsystem = "windows"]

use std::path::PathBuf;

use clap::{Arg, Command, value_parser};
use relm4::RelmApp;
use roomware::config;
use roomware::hints;
use roomware::master::GameMasterWindow;

fn main() {
    let matches = Command::new("roomware")
        .about("Escape room game master and player displays")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .help("TOML file to read the configuration from"),
        )
        .arg(
            Arg::new("hints")
                .short('i')
                .long("hints")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .help("TOML file to read the pre-written hints from"),
        )
        .get_matches();

    config::set_path(
        matches
            .get_one::<PathBuf>("config")
            .cloned()
            .or_else(|| desktop_file("config.toml")),
    );
    hints::set_path(
        matches
            .get_one::<PathBuf>("hints")
            .cloned()
            .or_else(|| desktop_file("hints.toml")),
    );

    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor").with_args(Vec::new());

    // Stops annoying empty hint entry box backspace noise on windows.
    if let Some(settings) = relm4::gtk::Settings::default() {
        settings.set_gtk_error_bell(false);
    }

    app.run::<GameMasterWindow>(());
}

/// Falls back to a file on the current user's desktop when no `-c`/`-i` flag
/// is passed.
fn desktop_file(name: &str) -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    let path = PathBuf::from(home).join("Desktop").join(name);

    path.is_file().then_some(path)
}
