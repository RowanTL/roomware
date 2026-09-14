use gtk::prelude::{
    CastNone, DisplayExt, GtkWindowExt, ListModelExt, MediaStreamExt, OrientableExt, WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::timer::format_time;

const DEFAULT_PLAYER_BACKGROUND: &str = "BF9560";
const VIDEO_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/video.mp4");

pub struct Player {
    remaining: u32,
    playing_video: bool,
    video_stream: Option<gtk::MediaStream>,
}

#[derive(Debug)]
pub enum PlayerMsg {
    UpdateTime(u32),
    PlayVideo,
    VideoFinished,
}

#[relm4::component(pub)]
impl SimpleComponent for Player {
    type Init = ();

    type Input = PlayerMsg;
    type Output = ();

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
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_vexpand: true,

                    gtk::Label {
                        #[watch]
                        set_label: &format_time(model.remaining),
                        set_margin_all: 5,
                    }
                },

                #[name = "video"]
                gtk::Video::for_filename(Some(VIDEO_PATH)) {
                    #[watch]
                    set_visible: model.playing_video,
                    set_autoplay: false,
                    set_loop: false,
                    set_hexpand: true,
                    set_vexpand: true,
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
                stream_to_play.seek(0);
                stream_to_play.play();
            });

            model.video_stream = Some(stream);
        }

        apply_background_style();
        place_on_secondary_monitor(&root);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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
        }
    }
}

fn apply_background_style() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let provider = gtk::CssProvider::new();
    provider.load_from_string(
        format!(
            "window.player-window {{ background-color: #{}; }}",
            DEFAULT_PLAYER_BACKGROUND
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
