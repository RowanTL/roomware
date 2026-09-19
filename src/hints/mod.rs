use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::utils::base_dir;

pub mod control;
pub mod menu;

pub const DEFAULT_HINT_AMT: u8 = 5;

static HINTS_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Has to be called before the menu is built in main.
pub fn set_path(path: Option<PathBuf>) {
    let _ = HINTS_PATH.set(path);
}

/// The hints file to read, if there is one.
pub(crate) fn path() -> Option<&'static Path> {
    HINTS_PATH
        .get()
        .and_then(Option::as_ref)
        .map(PathBuf::as_path)
}

/// Finds the base of a hints.toml path.
pub(crate) fn base() -> &'static Path {
    static BASE: OnceLock<PathBuf> = OnceLock::new();

    BASE.get_or_init(|| base_dir(path()))
}

pub struct Hints {
    counter: u8,
}

#[derive(Debug)]
pub enum HintsMsg {
    Increment,
    Decrement,
}

#[derive(Debug)]
pub enum HintsOutput {
    CounterChanged(u8),
}

#[relm4::component(pub)]
impl SimpleComponent for Hints {
    type Init = u8;

    type Input = HintsMsg;
    type Output = HintsOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            gtk::Button {
                set_label: "Increment",
                connect_clicked => HintsMsg::Increment
            },

            gtk::Button::with_label("Decrement") {
                connect_clicked => HintsMsg::Decrement
            },
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Hints { counter };

        let widgets = view_output!();

        let _ = sender.output(HintsOutput::CounterChanged(model.counter));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            HintsMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            HintsMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }

        let _ = sender.output(HintsOutput::CounterChanged(self.counter));
    }
}
