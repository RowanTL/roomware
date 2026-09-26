//! This file holds the text entry box and the show/clear hint buttons.

use gtk::prelude::{BoxExt, ButtonExt, EditableExt, EntryExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

pub struct HintControl {
    entry: Option<gtk::Entry>,
}

#[derive(Debug)]
pub enum HintControlMsg {
    Show,
    Submit,
    Clear,
    SetText(String),
    ShowValue(String),
}

#[derive(Debug)]
pub enum HintControlOutput {
    ShowHint(String),
}

#[relm4::component(pub)]
impl SimpleComponent for HintControl {
    type Init = ();

    type Input = HintControlMsg;
    type Output = HintControlOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            #[name = "entry"]
            gtk::Entry {
                set_hexpand: true,
                connect_activate => HintControlMsg::Submit,
            },

            gtk::Button::with_label("Show Hint") {
                connect_clicked => HintControlMsg::Show,
            },

            gtk::Button::with_label("Clear Hint") {
                connect_clicked => HintControlMsg::Clear,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = HintControl { entry: None };

        let widgets = view_output!();

        model.entry = Some(widgets.entry.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        let Some(entry) = &self.entry else {
            return;
        };

        match msg {
            HintControlMsg::Show => {
                let _ = sender.output(HintControlOutput::ShowHint(entry.text().to_string()));
                entry.set_text("");
            }
            // Happens whenever user clicks enter on their keyboard
            // Apparently this functionality is baked into gtk's Entry widget
            HintControlMsg::Submit => {
                let hint = entry.text().to_string();

                // If hint box is empty, do nothing
                if hint.trim().is_empty() {
                    return;
                }

                let _ = sender.output(HintControlOutput::ShowHint(hint));
                entry.set_text("");
            }
            HintControlMsg::Clear => {
                let _ = sender.output(HintControlOutput::ShowHint(String::new()));
                entry.set_text("");
            }
            HintControlMsg::SetText(text) => {
                entry.set_text(&text);
            }
            HintControlMsg::ShowValue(text) => {
                entry.set_text(&text);
                let _ = sender.output(HintControlOutput::ShowHint(text));
                entry.set_text("");
            }
        }
    }
}
