//! This file serves as a hub for various useful options that don't fit in anywhere else.

use gtk::glib::object::Cast;
use gtk::prelude::{ApplicationExt, BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, Sender, SimpleComponent, gtk};

use crate::player::PlayerMsg;
use crate::rooms;
use crate::server::{self, Server};
use crate::settings::{self, Settings};
use crate::update::{self, Update};
use crate::utils::busy_popup;

pub struct Misc {
    root: gtk::Box,
    force_loss: bool,
    settings: Settings,
    server: Server,
    busy: Option<gtk::Window>,
}

#[derive(Debug)]
pub enum MiscMsg {
    ShowSettings,
    SaveSettings(Settings),
    CheckForUpdate,
    UpdateChecked(Result<Option<Update>, String>),
    InstallUpdate(Update),
    UpdateStarted(Result<(), String>),
    UpdateHints,
    HintsUpdated(Result<(), String>),
    ShowVersion,
    ToggleForceLoss,
    Reset,
    Quit,
}

#[derive(Debug)]
pub enum MiscOutput {
    ForceLoss(bool),
    Reset,
    HintsUpdated,
}

#[relm4::component(pub)]
impl SimpleComponent for Misc {
    /// The web server tells the player when to start the video.
    type Init = Sender<PlayerMsg>;

    type Input = MiscMsg;
    type Output = MiscOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_valign: gtk::Align::Start,

            gtk::Button::with_label("Settings") {
                connect_clicked => MiscMsg::ShowSettings,
            },

            gtk::Button::with_label("About") {
                connect_clicked => MiscMsg::ShowVersion,
            },

            gtk::Button {
                #[watch]
                set_label: &format!("Force Loss: {}", if model.force_loss { "True" } else { "False" }),
                connect_clicked => MiscMsg::ToggleForceLoss,
            },

            gtk::Button::with_label("Reset") {
                connect_clicked => MiscMsg::Reset,
            },

            gtk::Button::with_label("Quit") {
                connect_clicked => MiscMsg::Quit,
            },
        }
    }

    fn init(
        player: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let settings = Settings::load(&settings::path());
        let mut server = Server::new(player);
        if let Err(error) = server.apply(&settings) {
            eprintln!("roomware: web server not started: {error}");
        }

        let model = Misc {
            root: root.clone(),
            force_loss: false,
            settings,
            server,
            busy: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            MiscMsg::ShowSettings => {
                let status = match server::lan_url().filter(|_| self.server.is_running()) {
                    Some(url) => format!("Running at {url}"),
                    None if self.server.is_running() => "Running".to_string(),
                    None => "Not running".to_string(),
                };
                let update_sender = sender.clone();
                let hints_sender = sender.clone();
                settings::show_dialog(
                    self.window().as_ref(),
                    &self.settings,
                    &status,
                    move |settings| sender.input(MiscMsg::SaveSettings(settings)),
                    move || update_sender.input(MiscMsg::CheckForUpdate),
                    move || hints_sender.input(MiscMsg::UpdateHints),
                );
            }
            MiscMsg::UpdateHints => {
                self.show_busy("Checking GitHub for updated hints...");
                std::thread::spawn(move || {
                    sender.input(MiscMsg::HintsUpdated(rooms::fetch_hints()));
                });
            }
            MiscMsg::HintsUpdated(result) => {
                self.hide_busy();
                match result {
                    Ok(()) => {
                        let _ = sender.output(MiscOutput::HintsUpdated);
                        self.alert("Hints updated", "The hint menu now shows the latest hints.");
                    }
                    Err(error) => self.alert("Could not update hints", &error),
                }
            }
            MiscMsg::SaveSettings(settings) => {
                let path = settings::path();
                let saved = settings
                    .save(&path)
                    .map_err(|error| format!("Could not write {}: {error}", path.display()));
                let applied = self
                    .server
                    .apply(&settings)
                    .map_err(|error| format!("Could not start the web server: {error}"));
                self.settings = settings;

                if let Err(error) = saved.and(applied) {
                    self.alert("Settings", &error);
                }
            }
            MiscMsg::CheckForUpdate => {
                self.show_busy("Checking for updates...");
                std::thread::spawn(move || sender.input(MiscMsg::UpdateChecked(update::check())));
            }
            MiscMsg::UpdateChecked(result) => {
                self.hide_busy();
                match result {
                    Ok(Some(update)) if cfg!(windows) => {
                        let dialog = gtk::AlertDialog::builder()
                            .message(format!("RoomWare {} is available", update.version))
                            .detail(
                                "RoomWare will close, install the update, and open again. \
                                 Windows asks for permission first.",
                            )
                            .buttons(["Cancel", "Update"])
                            .cancel_button(0)
                            .default_button(1)
                            .modal(true)
                            .build();
                        dialog.choose(
                            self.window().as_ref(),
                            gtk::gio::Cancellable::NONE,
                            move |choice| {
                                if matches!(choice, Ok(1)) {
                                    sender.input(MiscMsg::InstallUpdate(update));
                                }
                            },
                        );
                    }
                    Ok(Some(update)) => self.alert(
                        &format!("RoomWare {} is available", update.version),
                        "Updates can only be installed from inside RoomWare on Windows.",
                    ),
                    Ok(None) => self.alert(
                        "RoomWare is up to date",
                        concat!("This is version ", env!("CARGO_PKG_VERSION"), "."),
                    ),
                    Err(error) => self.alert("Could not check for updates", &error),
                }
            }
            MiscMsg::InstallUpdate(update) => {
                self.show_busy("Downloading the update...");
                std::thread::spawn(move || {
                    sender.input(MiscMsg::UpdateStarted(update::install(&update)));
                });
            }
            MiscMsg::UpdateStarted(result) => {
                self.hide_busy();
                match result {
                    // The installer waits for RoomWare to close before replacing it.
                    Ok(()) => relm4::main_application().quit(),
                    Err(error) => self.alert("Could not install the update", &error),
                }
            }
            MiscMsg::ShowVersion => {
                let logo = gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(
                    include_bytes!("../nix/escapeonmain.png"),
                ))
                .ok();

                let mut comments = String::from("Author: Rowan Torbitzky-Lane");
                if let Some(room) = rooms::name() {
                    comments.push_str(&format!("\nRoom: {room}"));
                }
                if let Some(url) = server::lan_url() {
                    comments.push_str(&format!("\nWeb page: {url}"));
                }

                let dialog = gtk::AboutDialog::builder()
                    .program_name("RoomWare")
                    .version(env!("CARGO_PKG_VERSION"))
                    .comments(comments)
                    .website("https://rowantl.github.io")
                    .website_label("rowantl.github.io")
                    .modal(true)
                    .hide_on_close(false)
                    .build();
                dialog.set_logo(logo.as_ref());
                dialog.set_transient_for(self.window().as_ref());
                dialog.present();
            }
            MiscMsg::ToggleForceLoss => {
                self.force_loss = !self.force_loss;
                let _ = sender.output(MiscOutput::ForceLoss(self.force_loss));
            }
            MiscMsg::Reset => {
                // The stats worker clears its own copy on reset, so no ForceLoss output here
                self.force_loss = false;
                let _ = sender.output(MiscOutput::Reset);
            }
            MiscMsg::Quit => relm4::main_application().quit(),
        }
    }
}

impl Misc {
    fn window(&self) -> Option<gtk::Window> {
        self.root
            .root()
            .and_then(|root| root.downcast::<gtk::Window>().ok())
    }

    fn alert(&self, message: &str, detail: &str) {
        gtk::AlertDialog::builder()
            .message(message)
            .detail(detail)
            .build()
            .show(self.window().as_ref());
    }

    fn show_busy(&mut self, text: &str) {
        let popup = busy_popup(text);
        popup.set_transient_for(self.window().as_ref());
        popup.present();
        self.busy = Some(popup);
    }

    fn hide_busy(&mut self) {
        if let Some(popup) = self.busy.take() {
            popup.destroy();
        }
    }
}
