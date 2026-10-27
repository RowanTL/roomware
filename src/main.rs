// Keeps Windows from opening a console window beside the game master's.
#![windows_subsystem = "windows"]

use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};
use relm4::RelmApp;
use roomware::master::GameMasterWindow;
use roomware::{config, hints, rooms};

fn main() {
    let matches = Command::new("roomware")
        .about("Escape room game master and player displays")
        .arg(
            Arg::new("room")
                .value_name("ROOM")
                .required(true)
                .help("Room to download and run, named as in the repo's rooms folder"),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Download every file of the room again, even unchanged ones"),
        )
        .arg(
            Arg::new("local")
                .short('l')
                .long("local")
                .action(ArgAction::SetTrue)
                .conflicts_with("force")
                .help("Read the room from ./rooms without downloading anything"),
        )
        .get_matches();

    let room = matches.get_one::<String>("room").expect("room is required");
    let dir = if matches.get_flag("local") {
        PathBuf::from("rooms").join(room)
    } else {
        let dir = rooms::installed_dir().join(room);
        rooms::sync(room, &dir, matches.get_flag("force"));
        dir
    };

    if !dir.is_dir() {
        eprintln!("roomware: no room at {}", dir.display());
        std::process::exit(1);
    }

    config::set_path(Some(dir.join(rooms::CONFIG_FILE)));
    hints::set_path(Some(dir.join(rooms::HINTS_FILE)));

    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor").with_args(Vec::new());

    // Stops annoying empty hint entry box backspace noise on windows.
    if let Some(settings) = relm4::gtk::Settings::default() {
        settings.set_gtk_error_bell(false);
    }

    app.run::<GameMasterWindow>(());
}
