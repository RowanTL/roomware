// Keeps Windows from opening a console window beside the game master's.
#![windows_subsystem = "windows"]

use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};
use gtk::glib;
use gtk::prelude::{BoxExt, GtkWindowExt};
use relm4::{RelmApp, gtk};
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
        sync_with_popup(room, &dir, matches.get_flag("force"));
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

/// Fetches the room behind a popup, since the main window isn't up yet.
fn sync_with_popup(room: &str, dir: &Path, force: bool) {
    let fetch = {
        let (room, dir) = (room.to_string(), dir.to_path_buf());
        move || {
            rooms::sync(&room, &dir, force);
            rooms::update_hints();
        }
    };

    if gtk::init().is_err() {
        return fetch();
    }

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();
    content.append(&gtk::Spinner::builder().spinning(true).build());
    content.append(&gtk::Label::new(Some(
        "Checking GitHub for updated hints...",
    )));

    let popup = gtk::Window::builder()
        .title("RoomWare")
        .resizable(false)
        .deletable(false)
        .child(&content)
        .build();
    popup.present();

    let main_loop = glib::MainLoop::new(None, false);
    let quit = main_loop.clone();
    std::thread::spawn(move || {
        fetch();
        // Queued rather than called here, so it can't land before the loop runs.
        glib::MainContext::default().invoke(move || quit.quit());
    });
    main_loop.run();

    popup.destroy();
}
