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

    config::set_path(matches.get_one::<PathBuf>("config").cloned());
    hints::set_path(matches.get_one::<PathBuf>("hints").cloned());

    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor").with_args(Vec::new());
    app.run::<GameMasterWindow>(());
}
