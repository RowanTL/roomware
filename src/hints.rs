use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

pub const DEFAULT_HINT_AMT: u8 = 5;

pub struct Hints {
    counter: u8,
}

#[derive(Debug)]
pub enum HintsMsg {
    Increment,
    Decrement,
}

#[relm4::component(pub)]
impl SimpleComponent for Hints {
    type Init = u8;

    type Input = HintsMsg;
    type Output = ();

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

            gtk::Label {
                #[watch]
                set_label: &format!("Hints Remaining: {}", model.counter),
                set_margin_all: 5,
            }
        }
    }

    fn init(
        counter: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Hints { counter };

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            HintsMsg::Increment => {
                self.counter = self.counter.wrapping_add(1);
            }
            HintsMsg::Decrement => {
                self.counter = self.counter.wrapping_sub(1);
            }
        }
    }
}
