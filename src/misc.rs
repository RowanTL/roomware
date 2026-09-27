//! This file serves as a hub for various useful options that don't fit in anywhere else.

use gtk::glib::object::Cast;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

pub struct Misc {
    root: gtk::Box,
    force_loss: bool,
}

#[derive(Debug)]
pub enum MiscMsg {
    ShowVersion,
    ToggleForceLoss,
    // The stats worker clears its own copy on reset, so this only updates the label
    ResetForceLoss,
}

#[derive(Debug)]
pub enum MiscOutput {
    ForceLoss(bool),
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

            gtk::Button::with_label("Show Version") {
                connect_clicked => MiscMsg::ShowVersion,
            },

            gtk::Button {
                #[watch]
                set_label: &format!("Force Loss: {}", if model.force_loss { "True" } else { "False" }),
                connect_clicked => MiscMsg::ToggleForceLoss,
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

                gtk::AlertDialog::builder()
                    .message("RoomWare")
                    .detail(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .modal(true)
                    .build()
                    .show(window.as_ref());
            }
            MiscMsg::ToggleForceLoss => {
                self.force_loss = !self.force_loss;
                let _ = sender.output(MiscOutput::ForceLoss(self.force_loss));
            }
            MiscMsg::ResetForceLoss => {
                self.force_loss = false;
            }
        }
    }
}
