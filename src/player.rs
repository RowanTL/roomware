use gtk::prelude::{
    CastNone, DisplayExt, GtkWindowExt, ListModelExt, MediaStreamExt, OrientableExt, WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::timer::format_time;

const DEFAULT_PLAYER_BACKGROUND: &str = "BF9560";
const DEFAULT_TIMER_FONT_SIZE: usize = 100;
const DEFAULT_HINT_FONT_SIZE: usize = 70;
const DEFAULT_HINTS_REMAINING_FONT_SIZE: usize = 50;
const VIDEO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/video.mp4");

pub struct Player {
    remaining: u32,
    playing_video: bool,
    video_stream: Option<gtk::MediaStream>,
    hint: String,
    show_hint: bool,
    hints_remaining: u8,
}

#[derive(Debug)]
pub enum PlayerMsg {
    UpdateTime(u32),
    PlayVideo,
    VideoFinished,
    ShowHint(String),
    UpdateHintsRemaining(u8),
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub remaining: u32,
    pub playing_video: bool,
    pub hint: String,
    pub show_hint: bool,
    pub hints_remaining: u8,
}

#[derive(Debug)]
pub enum PlayerOutput {
    StateChanged(PlayerState),
}

#[relm4::component(pub)]
impl SimpleComponent for Player {
    type Init = ();

    type Input = PlayerMsg;
    type Output = PlayerOutput;

    view! {
        gtk::Window {
            set_title: Some("RoomWare - Player Display"),
            set_default_width: 300,
            set_default_height: 100,
            add_css_class: "player-window",

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Box {
                    #[watch]
                    set_visible: !model.playing_video,
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,

                    gtk::Label {
                        #[watch]
                        set_label: &format_time(model.remaining),
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Start,
                        set_margin_all: 5,
                        add_css_class: "timer-label",
                    },

                    gtk::Box {
                        #[watch]
                        set_visible: model.show_hint,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_vexpand: true,

                        gtk::Label {
                            #[watch]
                            set_label: &model.hint,
                            set_margin_all: 5,
                            add_css_class: "message-label",
                        }
                    }
                },

                #[name = "video"]
                gtk::Video::for_filename(Some(VIDEO_PATH)) {
                    #[watch]
                    set_visible: model.playing_video,
                    set_autoplay: true,
                    set_loop: false,
                    set_hexpand: true,
                    set_vexpand: true,
                },

                gtk::Label {
                    #[watch]
                    set_label: &format!("Hints Remaining: {}", model.hints_remaining),
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::End,
                    set_margin_all: 5,
                    add_css_class: "hints-remaining-label",
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Player {
            remaining: 0,
            playing_video: false,
            video_stream: None,
            hint: String::new(),
            show_hint: false,
            hints_remaining: 0,
        };

        let widgets = view_output!();

        if let Some(stream) = widgets.video.media_stream() {
            let input_sender = sender.input_sender().clone();
            stream.connect_ended_notify(move |stream| {
                if stream.is_ended() {
                    input_sender.emit(PlayerMsg::VideoFinished);
                }
            });

            let stream_to_play = stream.clone();
            widgets.video.connect_map(move |_| {
                stream_to_play.seek(0); // frame 0 of video
                stream_to_play.play();
            });

            model.video_stream = Some(stream);
        }

        apply_css_style();
        place_on_secondary_monitor(&root);

        let _ = sender.output(PlayerOutput::StateChanged(model.to_state()));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PlayerMsg::UpdateTime(remaining) => {
                self.remaining = remaining;
            }
            PlayerMsg::PlayVideo => {
                self.playing_video = true;
            }
            PlayerMsg::VideoFinished => {
                self.playing_video = false;
                if let Some(stream) = &self.video_stream {
                    stream.pause();
                }
            }
            PlayerMsg::ShowHint(hint) => {
                let hint = hint.trim().to_string();
                self.show_hint = !hint.is_empty();
                self.hint = hint;
            }
            PlayerMsg::UpdateHintsRemaining(hints_remaining) => {
                self.hints_remaining = hints_remaining;
            }
        }

        let _ = sender.output(PlayerOutput::StateChanged(self.to_state()));
    }
}

impl Player {
    fn to_state(&self) -> PlayerState {
        PlayerState {
            remaining: self.remaining,
            playing_video: self.playing_video,
            hint: self.hint.clone(),
            show_hint: self.show_hint,
            hints_remaining: self.hints_remaining,
        }
    }
}

fn apply_css_style() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let provider = gtk::CssProvider::new();
    provider.load_from_string(
        format!(
            "window.player-window {{ background-color: #{}; }} label.timer-label {{ font-size: {}px; }} label.message-label {{ font-size: {}px; }} label.hints-remaining-label {{ font-size: {}px; }}",
            DEFAULT_PLAYER_BACKGROUND,
            DEFAULT_TIMER_FONT_SIZE,
            DEFAULT_HINT_FONT_SIZE,
            DEFAULT_HINTS_REMAINING_FONT_SIZE
        )
        .as_str(),
    );
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn place_on_secondary_monitor(window: &gtk::Window) {
    // https://doc.rust-lang.org/beta/rust-by-example/flow_control/let_else.html
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let monitors = display.monitors();
    let monitor = monitors
        .item(1) // 1 represents the second monitor
        .and_downcast::<gtk::gdk::Monitor>()
        .or_else(|| monitors.item(0).and_downcast::<gtk::gdk::Monitor>());

    if let Some(monitor) = monitor {
        window.fullscreen_on_monitor(&monitor);
    }
}
