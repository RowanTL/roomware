use gtk::prelude::{BoxExt, Cast, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::config::config;
use crate::player::{
    DEFAULT_HINT_FONT_SIZE, DEFAULT_HINTS_REMAINING_FONT_SIZE, DEFAULT_TIMER_FONT_SIZE,
    HINT_WRAP_WIDTH_CHARS, PlayerState, TIME_REMAINING_GAP, TIME_REMAINING_LINE_HEIGHT,
    TIMER_LINE_HEIGHT, background_css, fit_font_size,
};
use crate::timer::{Outcome, format_time};
use crate::utils::apply_css_style;

const PREVIEW_WIDTH: i32 = 300;
const PREVIEW_HEIGHT: i32 = 100;
const PREVIEW_FONT_SCALE_DIVISOR: usize = 1;

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

            gtk::Box {
                #[watch]
                set_visible: !model.state.playing_video,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: TIME_REMAINING_GAP,

                gtk::Label {
                    set_label: "Time Remaining:",
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    add_css_class: "time-remaining-label-preview",
                },

                gtk::Label {
                    #[watch]
                    set_label: &format_time(model.state.remaining),
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    add_css_class: "timer-label-preview",
                    #[watch]
                    set_class_active: ("win", model.state.outcome == Outcome::Won),
                    #[watch]
                    set_class_active: ("lose", model.state.outcome == Outcome::Lost),
                },
            },

            #[name = "hint_area"]
            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vscrollbar_policy: gtk::PolicyType::External,

                gtk::Box {
                    set_vexpand: true,
                    set_hexpand: true,
                    set_halign: gtk::Align::Fill,
                    set_valign: gtk::Align::Center,

                    #[name = "hint_label"]
                    gtk::Label {
                        #[watch]
                        set_visible: !model.state.playing_video
                            && model.state.show_hint
                            && model.state.hint_image.is_none(),
                        #[watch]
                        set_label: &model.state.hint,
                        set_hexpand: true,
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                        set_max_width_chars: HINT_WRAP_WIDTH_CHARS,
                        set_justify: gtk::Justification::Center,
                        add_css_class: "message-label-preview",
                    },

                    gtk::Picture {
                        #[watch]
                        set_visible: !model.state.playing_video
                            && model.state.show_hint
                            && model.state.hint_image.is_some(),
                        #[watch]
                        set_paintable: model.state.hint_image.as_ref(),
                        set_hexpand: true,
                    }
                }
            },

            gtk::Box {
                #[watch]
                set_visible: !model.state.playing_video,
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,

                gtk::Label {
                    set_label: "Hints Remaining: ",
                    add_css_class: "hints-remaining-label-preview",
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.state.hints_remaining.to_string(),
                    add_css_class: "hints-amount-label-preview",
                },
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
                outcome: Outcome::Undecided,
                playing_video: false,
                hint: String::new(),
                hint_image: None,
                show_hint: false,
                hints_remaining: 0,
            },
        };

        let widgets = view_output!();

        apply_css_style(&format!(
            "box.player-preview-box {{ {background} }} \
             box.player-preview-box scrolledwindow, \
             box.player-preview-box scrolledwindow viewport {{ background: none; }} \
             box.player-preview-box label.timer-label-preview {{ font-size: {timer_size}px; line-height: {timer_line_height}; color: #{clock}; }} \
             box.player-preview-box label.timer-label-preview.win {{ color: #{win}; }} \
             box.player-preview-box label.timer-label-preview.lose {{ color: #{lose}; }} \
             box.player-preview-box label.message-label-preview {{ font-size: {hint_size}px; color: #{hint}; }} \
             box.player-preview-box label.time-remaining-label-preview {{ line-height: {time_remaining_line_height}; color: #{clock_remaining}; }} \
             box.player-preview-box label.time-remaining-label-preview, \
             box.player-preview-box label.hints-remaining-label-preview, \
             box.player-preview-box label.hints-amount-label-preview {{ font-size: {remaining_size}px; }} \
             box.player-preview-box label.hints-remaining-label-preview {{ color: #{hint_remaining}; }} \
             box.player-preview-box label.hints-amount-label-preview {{ color: #{hint_amount}; }}",
            background = background_css(),
            timer_size = DEFAULT_TIMER_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
            timer_line_height = TIMER_LINE_HEIGHT,
            time_remaining_line_height = TIME_REMAINING_LINE_HEIGHT,
            hint_size = DEFAULT_HINT_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
            remaining_size = DEFAULT_HINTS_REMAINING_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
            clock = config().player_clock_color,
            clock_remaining = config().player_clock_remaining_color,
            hint = config().player_hint_color,
            hint_remaining = config().player_hint_remaining_color,
            hint_amount = config().player_hint_amount_color,
            win = config().player_win_color,
            lose = config().player_lose_color,
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

    fn post_view() {
        fit_font_size(
            &widgets.hint_label,
            widgets.hint_area.upcast_ref(),
            DEFAULT_HINT_FONT_SIZE / PREVIEW_FONT_SCALE_DIVISOR,
        );
    }
}
