//! This file serves as a hub for various useful options that don't fit in anywhere else.

use gtk::glib::object::Cast;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

pub struct Misc {
    root: gtk::Box,
}

#[derive(Debug)]
pub enum MiscMsg {
    ShowVersion,
}

#[relm4::component(pub)]
impl SimpleComponent for Misc {
    type Init = ();

    type Input = MiscMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,
            set_valign: gtk::Align::Start,

            gtk::Button::with_label("Show Version") {
                connect_clicked => MiscMsg::ShowVersion,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Misc { root: root.clone() };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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
        }
    }
}
