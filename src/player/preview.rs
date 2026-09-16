use gtk::prelude::{OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

use crate::player::{
    DEFAULT_HINT_FONT_SIZE, DEFAULT_HINTS_REMAINING_FONT_SIZE, DEFAULT_PLAYER_BACKGROUND,
    DEFAULT_TIMER_FONT_SIZE, PlayerState,
};
use crate::timer::format_time;
use crate::utils::apply_css_style;

const PREVIEW_WIDTH: i32 = 300;
const PREVIEW_HEIGHT: i32 = 100;

// Player's fonts are sized for a fullscreen monitor; shrink them to fit this little box.
const PREVIEW_FONT_SCALE_DIVISOR: usize = 2;

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
            set_width_request: PREVIEW_WIDTH,
            set_height_request: PREVIEW_HEIGHT,
            set_overflow: gtk::Overflow::Hidden,
            add_css_class: "player-preview-box",

            gtk::Label {
                #[watch]
                set_visible: model.state.playing_video,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                set_label: "Intro Playing",
                add_css_class: "timer-label-preview",
            },

            gtk::Label {
                #[watch]
                set_visible: !model.state.playing_video,
                #[watch]
                set_label: &format_time(model.state.remaining),
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Start,
                add_css_class: "timer-label-preview",
            },

            gtk::Box {
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    #[watch]
                    set_visible: !model.state.playing_video && model.state.show_hint,
                    #[watch]
                    set_label: &model.state.hint,
                    add_css_class: "message-label-preview",
                }
            },

            gtk::Label {
                #[watch]
                set_label: &format!("Hints Remaining: {}", model.state.hints_remaining),
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
                add_css_class: "hints-remaining-label-preview",
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

        apply_css_style(&format!(
            "box.player-preview-box {{ background-color: #{}; }} \
             box.player-preview-box label.timer-label-preview {{ font-size: {}px; color: #1A1A1A; }} \
             box.player-preview-box label.message-label-preview {{ font-size: {}px; color: #1A1A1A; }} \
             box.player-preview-box label.hints-remaining-label-preview {{ font-size: {}px; color: #1A1A1A; }}",
            DEFAULT_PLAYER_BACKGROUND,
            DEFAULT_TIMER_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
            DEFAULT_HINT_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
            DEFAULT_HINTS_REMAINING_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR
        ));

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
