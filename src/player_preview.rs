use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::player::PlayerState;
use crate::timer::format_time;

pub struct PlayerPreview {
    state: PlayerState,
}

#[derive(Debug)]
pub enum PlayerPreviewMsg {
    Sync(PlayerState),
}

#[relm4::component(pub)]
impl SimpleComponent for PlayerPreview {
    type Init = ();

    type Input = PlayerPreviewMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            gtk::Label {
                #[watch]
                set_visible: model.state.playing_video,
                set_label: "Intro Playing",
            },

            gtk::Label {
                #[watch]
                set_visible: !model.state.playing_video,
                #[watch]
                set_label: &format_time(model.state.remaining),
            },

            gtk::Label {
                #[watch]
                set_visible: !model.state.playing_video && model.state.show_hint,
                #[watch]
                set_label: &model.state.hint,
            },

            gtk::Label {
                #[watch]
                set_label: &format!("Hints Remaining: {}", model.state.hints_remaining),
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PlayerPreview {
            state: PlayerState {
                remaining: 0,
                playing_video: false,
                hint: String::new(),
                show_hint: false,
                hints_remaining: 0,
            },
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PlayerPreviewMsg::Sync(state) => {
                self.state = state;
            }
        }
    }
}
