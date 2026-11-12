//! This file serves as a hub for various useful options that don't fit in anywhere else.

use gtk::glib::object::Cast;
use gtk::prelude::{
    ApplicationExt, BoxExt, ButtonExt, EditableExt, GridExt, GtkWindowExt, OrientableExt, WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, Sender, SimpleComponent, gtk};

use crate::player::PlayerMsg;
use crate::server::settings::{self, Settings};
use crate::server::{self, Server};

pub struct Misc {
    root: gtk::Box,
    force_loss: bool,
    settings: Settings,
    server: Server,
}

#[derive(Debug)]
pub enum MiscMsg {
    ShowSettings,
    SaveSettings(Settings),
    ShowVersion,
    ToggleForceLoss,
    Reset,
    Quit,
}

#[derive(Debug)]
pub enum MiscOutput {
    ForceLoss(bool),
    Reset,
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
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            MiscMsg::ShowSettings => {
                let password = gtk::Entry::builder()
                    .text(self.settings.password.as_str())
                    .hexpand(true)
                    .width_chars(30)
                    .build();
                let active = gtk::Switch::builder()
                    .active(self.settings.should_serve())
                    .sensitive(!self.settings.password.is_empty())
                    .halign(gtk::Align::Start)
                    .build();
                let status = match server::lan_url().filter(|_| self.server.is_running()) {
                    Some(url) => format!("Running at {url}"),
                    None if self.server.is_running() => "Running".to_string(),
                    None => "Not running".to_string(),
                };
                let save = gtk::Button::with_label("Save");

                let grid = gtk::Grid::builder()
                    .row_spacing(10)
                    .column_spacing(10)
                    .margin_top(10)
                    .margin_bottom(10)
                    .margin_start(10)
                    .margin_end(10)
                    .build();
                let label = |text: &str| gtk::Label::builder().label(text).xalign(0.0).build();
                grid.attach(&label("Web page password"), 0, 0, 1, 1);
                grid.attach(&password, 1, 0, 1, 1);
                grid.attach(&label("Web server"), 0, 1, 1, 1);
                grid.attach(&active, 1, 1, 1, 1);
                grid.attach(&label(&status), 0, 2, 2, 1);
                grid.attach(&save, 0, 3, 2, 1);

                let dialog = gtk::Window::builder()
                    .title("Settings")
                    .modal(true)
                    .resizable(false)
                    .child(&grid)
                    .build();
                dialog.set_transient_for(self.window().as_ref());

                // The server can't be on without a password.
                let switch = active.clone();
                password.connect_changed(move |password| {
                    let blank = password.text().trim().is_empty();
                    switch.set_sensitive(!blank);
                    if blank {
                        switch.set_active(false);
                    }
                });

                let window = dialog.clone();
                save.connect_clicked(move |_| {
                    sender.input(MiscMsg::SaveSettings(Settings {
                        password: password.text().trim().to_string(),
                        is_active: active.is_active(),
                    }));
                    window.close();
                });

                dialog.present();
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
                    gtk::AlertDialog::builder()
                        .message("Settings")
                        .detail(error)
                        .build()
                        .show(self.window().as_ref());
                }
            }
            MiscMsg::ShowVersion => {
                let logo = gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(
                    include_bytes!("../nix/escapeonmain.png"),
                ))
                .ok();

                let mut comments = String::from("Author: Rowan Torbitzky-Lane");
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
}
