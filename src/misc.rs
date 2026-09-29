//! This file serves as a hub for various useful options that don't fit in anywhere else.

use gtk::glib::object::Cast;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

pub struct Misc {
    root: gtk::Box,
    force_loss: bool,
}

#[derive(Debug)]
pub enum MiscMsg {
    ShowVersion,
    ToggleForceLoss,
    Reset,
}

#[derive(Debug)]
pub enum MiscOutput {
    ForceLoss(bool),
    Reset,
}

#[relm4::component(pub)]
impl SimpleComponent for Misc {
    type Init = ();

    type Input = MiscMsg;
    type Output = MiscOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_valign: gtk::Align::Start,

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
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Misc {
            root: root.clone(),
            force_loss: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            MiscMsg::ShowVersion => {
                let window = self
                    .root
                    .root()
                    .and_then(|root| root.downcast::<gtk::Window>().ok());

                let logo = gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(
                    include_bytes!("../nix/escapeonmain.png"),
                ))
                .ok();

                let dialog = gtk::AboutDialog::builder()
                    .program_name("RoomWare")
                    .version(env!("CARGO_PKG_VERSION"))
                    .comments("Author: Rowan Torbitzky-Lane")
                    .website("https://rowantl.github.io")
                    .website_label("rowantl.github.io")
                    .modal(true)
                    .hide_on_close(false)
                    .build();
                dialog.set_logo(logo.as_ref());
                dialog.set_transient_for(window.as_ref());
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
        }
    }
}
