use gtk::prelude::{CastNone, DisplayExt, GtkWindowExt, ListModelExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::timer::format_time;

const DEFAULT_PLAYER_BACKGROUND: &str = "BF9560";

pub struct Player {
    remaining: u32,
}

#[derive(Debug)]
pub enum PlayerMsg {
    UpdateTime(u32),
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
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    #[watch]
                    set_label: &format_time(model.remaining),
                    set_margin_all: 5,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Player { remaining: 0 };

        let widgets = view_output!();

        apply_background_style();
        place_on_secondary_monitor(&root);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            PlayerMsg::UpdateTime(remaining) => {
                self.remaining = remaining;
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
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };

    let monitors = display.monitors();
    let monitor = monitors
        .item(1)
        .and_downcast::<gtk::gdk::Monitor>()
        .or_else(|| monitors.item(0).and_downcast::<gtk::gdk::Monitor>());

    if let Some(monitor) = monitor {
        window.fullscreen_on_monitor(&monitor);
    }
}
