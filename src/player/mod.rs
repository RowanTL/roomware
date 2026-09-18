use std::sync::OnceLock;

use gtk::prelude::{
    Cast, CastNone, DisplayExt, GtkWindowExt, ListModelExt, MediaStreamExt, OrientableExt,
    WidgetExt, WidgetExtManual,
};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::config::config;
use crate::timer::format_time;
use crate::utils::{apply_css_style, load_image};

pub mod preview;

pub(crate) const DEFAULT_TIMER_FONT_SIZE: usize = 100;
pub(crate) const DEFAULT_HINT_FONT_SIZE: usize = 50;
pub(crate) const DEFAULT_HINTS_REMAINING_FONT_SIZE: usize = 50;
pub(crate) const HINT_WRAP_WIDTH_CHARS: i32 = 30;
pub(crate) const MIN_HINT_FONT_SIZE: usize = 16;
pub(crate) const HINT_MARGIN: i32 = 5;

pub struct Player {
    remaining: u32,
    playing_video: bool,
    video_stream: Option<gtk::MediaStream>,
    audio_stream: Option<gtk::MediaStream>,
    hint: String,
    // Set when the hint is a path to an image; `hint` is left empty.
    hint_image: Option<gtk::gdk::Texture>,
    show_hint: bool,
    hints_remaining: u8,
}

#[derive(Debug)]
pub enum PlayerMsg {
    UpdateTime(u32),
    PlayVideo,
    PlayAudio,
    VideoFinished,
    ShowHint(String),
    UpdateHintsRemaining(u8),
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub remaining: u32,
    pub playing_video: bool,
    pub hint: String,
    pub hint_image: Option<gtk::gdk::Texture>,
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

                    #[name = "hint_area"]
                    gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vscrollbar_policy: gtk::PolicyType::External,

                        gtk::Box {
                            set_halign: gtk::Align::Fill,
                            set_valign: gtk::Align::Center,
                            set_hexpand: true,
                            set_vexpand: true,

                            #[name = "hint_label"]
                            gtk::Label {
                                #[watch]
                                set_visible: model.show_hint && model.hint_image.is_none(),
                                #[watch]
                                set_label: &model.hint,
                                set_margin_all: HINT_MARGIN,
                                set_hexpand: true,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::WordChar,
                                set_max_width_chars: HINT_WRAP_WIDTH_CHARS,
                                set_justify: gtk::Justification::Center,
                                add_css_class: "message-label",
                            },

                            gtk::Picture {
                                #[watch]
                                set_visible: model.show_hint && model.hint_image.is_some(),
                                #[watch]
                                set_paintable: model.hint_image.as_ref(),
                                set_margin_all: HINT_MARGIN,
                                set_hexpand: true,
                            }
                        }
                    }
                },

                #[name = "video"]
                gtk::Video::for_filename(config().player_video.as_ref()) {
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
            audio_stream: load_audio(),
            hint: String::new(),
            hint_image: None,
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

        apply_css_style(&format!(
            "window.player-window {{ {background} }} \
             window.player-window scrolledwindow, \
             window.player-window scrolledwindow viewport {{ background: none; }} \
             label.timer-label {{ font-size: {timer_size}px; color: #{label}; }} \
             label.message-label {{ font-size: {hint_size}px; color: #{label}; }} \
             label.hints-remaining-label {{ font-size: {remaining_size}px; color: #{label}; }}",
            background = background_css(),
            timer_size = DEFAULT_TIMER_FONT_SIZE,
            hint_size = DEFAULT_HINT_FONT_SIZE,
            remaining_size = DEFAULT_HINTS_REMAINING_FONT_SIZE,
            label = config().player_label_color,
        ));

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
                // Without a video there is nothing to end the playback, so the
                // screen would sit on an empty video forever.
                self.playing_video = self.video_stream.is_some();
            }
            PlayerMsg::PlayAudio => {
                if let Some(stream) = &self.audio_stream {
                    if stream.is_seekable() {
                        stream.seek(0);
                    }
                    stream.play();
                }
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
                self.hint_image = load_image(&hint);
                // A picture stands on its own; don't put its path on screen too.
                self.hint = if self.hint_image.is_some() {
                    String::new()
                } else {
                    hint
                };
            }
            PlayerMsg::UpdateHintsRemaining(hints_remaining) => {
                self.hints_remaining = hints_remaining;
            }
        }

        let _ = sender.output(PlayerOutput::StateChanged(self.to_state()));
    }

    fn post_view() {
        fit_font_size(
            &widgets.hint_label,
            widgets.hint_area.upcast_ref(),
            DEFAULT_HINT_FONT_SIZE,
        );
    }
}

impl Player {
    /// Helps with updating the preview in the game master view.
    fn to_state(&self) -> PlayerState {
        PlayerState {
            remaining: self.remaining,
            playing_video: self.playing_video,
            hint: self.hint.clone(),
            hint_image: self.hint_image.clone(),
            show_hint: self.show_hint,
            hints_remaining: self.hints_remaining,
        }
    }
}

/// A label is always as tall as its text, so a long hint would grow until it
/// crowded out the labels around it. Shrink the font instead, down to whatever
/// size makes the wrapped hint fit the space `area` actually has.
///
/// This one is also vibe coded.
pub(crate) fn fit_font_size(label: &gtk::Label, area: &gtk::Widget, max_size: usize) {
    if fit_font_size_now(label, area, max_size) {
        return;
    }

    // Nothing is laid out yet, so there is no size to fit to; the frame clock
    // hands us one as soon as there is.
    let label = label.clone();
    let pending_area = area.clone();
    area.add_tick_callback(move |_, _| {
        if fit_font_size_now(&label, &pending_area, max_size) {
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
}

/// Returns whether `area` was laid out far enough to fit anything to.
///
/// This is AI generated. I have no idea what's going on in this function.
fn fit_font_size_now(label: &gtk::Label, area: &gtk::Widget, max_size: usize) -> bool {
    let width = area.width() - HINT_MARGIN * 2;
    let height = area.height() - HINT_MARGIN * 2;
    if width <= 0 || height <= 0 {
        return false;
    }

    let text = label.text();
    if text.is_empty() {
        label.set_attributes(None);
        return true;
    }

    let context = label.pango_context();
    let mut font = context.font_description().unwrap_or_default();
    let layout = gtk::pango::Layout::new(&context);
    layout.set_text(&text);
    layout.set_wrap(gtk::pango::WrapMode::WordChar);
    layout.set_width(width * gtk::pango::SCALE);

    let mut fits = |size: usize| {
        font.set_absolute_size((size * gtk::pango::SCALE as usize) as f64);
        layout.set_font_description(Some(&font));
        layout.pixel_size().1 <= height
    };

    // Taller text never fits where shorter text didn't, so binary search for the
    // largest size that still fits rather than measuring every size.
    let (mut smallest, mut largest) = (MIN_HINT_FONT_SIZE, max_size.max(MIN_HINT_FONT_SIZE));
    while smallest < largest {
        let size = smallest + (largest - smallest).div_ceil(2);
        if fits(size) {
            smallest = size;
        } else {
            largest = size - 1;
        }
    }

    let attributes = gtk::pango::AttrList::new();
    attributes.insert(gtk::pango::AttrSize::new_size_absolute(
        (smallest * gtk::pango::SCALE as usize) as i32,
    ));
    label.set_attributes(Some(&attributes));

    true
}

/// Loads the configured ping sound if there is one. Button does nothing
/// if there isn't.
fn load_audio() -> Option<gtk::MediaStream> {
    let path = config().master_noise.as_ref()?;

    Some(gtk::MediaFile::for_filename(path).upcast())
}

/// If the user doesn't configure a background image to use, this function
/// is called to create a basic background color instead.
pub(crate) fn background_css() -> &'static str {
    static BACKGROUND_CSS: OnceLock<String> = OnceLock::new();

    BACKGROUND_CSS.get_or_init(|| match &config().player_background_image {
        Some(uri) => format!(
            "background-image: url(\"{uri}\"); background-size: cover; background-position: center;"
        ),
        None => format!("background-color: #{};", config().player_background_color),
    })
}

// Checks if a second monitor exists and places the player window on it.
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
