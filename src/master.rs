//! The main screen the game master sees. Combines controls for the time, hints, audio, and
//! video.

use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, GtkWindowExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent, gtk,
};

use crate::config::config;
use crate::hints::control::{HintControl, HintControlMsg, HintControlOutput};
use crate::hints::menu::{HintMenu, HintMenuMsg, HintMenuOutput};
use crate::hints::{Hints, HintsOutput};
use crate::misc::Misc;
use crate::player::preview::{PlayerPreview, PlayerPreviewMsg};
use crate::player::{Player, PlayerMsg, PlayerOutput};
use crate::timer::{Timer, TimerOutput};
use crate::utils::apply_css_style;

pub struct GameMasterWindow {
    // dead_code here to keep these Controllers alive; their widgets are pulled out
    // into frames before the struct is built, so the fields themselves are never read
    #[allow(dead_code)]
    hints: Controller<Hints>,
    #[allow(dead_code)]
    timer: Controller<Timer>,
    #[allow(dead_code)]
    hint_control: Controller<HintControl>,
    #[allow(dead_code)]
    player_preview: Controller<PlayerPreview>,
    #[allow(dead_code)]
    hint_menu: Controller<HintMenu>,
    #[allow(dead_code)]
    player: Controller<Player>,
    #[allow(dead_code)]
    misc: Controller<Misc>,
}

#[relm4::component(pub)]
impl SimpleComponent for GameMasterWindow {
    type Init = ();

    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("RoomWare"),
            add_css_class: "main-window",

            #[name = "grid"]
            gtk::Grid {
                set_row_spacing: 5,
                set_column_spacing: 5,
                set_margin_all: 5,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let player_preview = PlayerPreview::builder().launch(()).detach();
        let player = Player::builder()
            .launch(())
            .forward(player_preview.sender(), |output| match output {
                PlayerOutput::StateChanged(state) => PlayerPreviewMsg::Sync(state),
            });
        let hints = Hints::builder()
            .launch(())
            .forward(player.sender(), |output| match output {
                HintsOutput::CounterChanged(remaining) => {
                    PlayerMsg::UpdateHintsRemaining(remaining)
                }
            });
        let hint_control = HintControl::builder()
            .launch(())
            .forward(player.sender(), |output| match output {
                HintControlOutput::ShowHint(hint) => PlayerMsg::ShowHint(hint),
            });
        let hint_menu = HintMenu::builder()
            .launch(())
            .forward(hint_control.sender(), |output| match output {
                HintMenuOutput::HintSelected(value) => HintControlMsg::SetText(value),
                HintMenuOutput::HintActivated(value) => HintControlMsg::ShowValue(value),
            });
        let player_sender = player.sender().clone();
        let hint_menu_sender = hint_menu.sender().clone();
        let timer = Timer::builder()
            .launch(())
            .connect_receiver(move |_, output| match output {
                TimerOutput::TimeChanged(remaining) => {
                    player_sender.emit(PlayerMsg::UpdateTime(remaining));
                }
                TimerOutput::OutcomeChanged(outcome) => {
                    player_sender.emit(PlayerMsg::UpdateOutcome(outcome));
                }
                TimerOutput::Reset => {
                    hint_menu_sender.emit(HintMenuMsg::ExpandAll);
                }
            });

        player.widget().present();

        let play_video_button = gtk::Button::with_label("Play Intro");
        let player_sender = player.sender().clone();
        play_video_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::PlayVideo);
        });

        let play_audio_button = gtk::Button::with_label("Ping Player");
        let player_sender = player.sender().clone();
        play_audio_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::PlayAudio);
        });

        let timer_frame = gtk::Frame::new(Some("Timer Controls"));
        timer_frame.set_child(Some(timer.widget()));

        let hint_frame = gtk::Frame::new(Some("Hint Controls"));
        hint_frame.set_child(Some(hints.widget()));

        let misc = Misc::builder().launch(()).detach();
        let misc_frame = gtk::Frame::new(Some("Misc"));
        misc_frame.set_child(Some(misc.widget()));

        let player_preview_frame = gtk::Frame::new(Some("Player View"));
        player_preview_frame.set_child(Some(player_preview.widget()));
        player_preview_frame.set_hexpand(true);
        player_preview_frame.add_css_class("player-preview-frame");

        let hint_menu_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        hint_menu_box.append(hint_menu.widget());
        hint_menu_box.append(hint_control.widget());

        let hint_menu_frame = gtk::Frame::new(Some("Hint Selection"));
        hint_menu_frame.set_child(Some(&hint_menu_box));

        let model = GameMasterWindow {
            hints,
            timer,
            hint_control,
            player_preview,
            hint_menu,
            player,
            misc,
        };

        let widgets = view_output!();

        let config = config();
        apply_css_style(&format!(
            "window.main-window {{ background-color: #{background}; }} \
             window.main-window frame {{ border: 2px solid #{frame_border}; padding: 6px; }} \
             window.main-window frame:not(.player-preview-frame) label {{ color: #{frame_label}; }} \
             window.main-window frame > label {{ color: #{frame_label}; font-weight: bold; }} \
             window.main-window expander-widget > box > title > expander {{ color: #{expander}; }} \
             window.main-window expander-widget > box > title:hover > expander {{ color: #{expander_hovered}; }} \
             window.main-window button {{ background-color: #{button_background}; color: #{button}; }} \
             window.main-window frame:not(.player-preview-frame) button label {{ color: #{button_label}; }} \
             window.main-window entry {{ background-color: #{entry_background}; color: #{entry_text}; }}",
            background = config.master_background,
            frame_border = config.frame_border,
            frame_label = config.frame_label,
            expander = config.expander_color,
            expander_hovered = config.expander_hovered_color,
            button_background = config.button_background_color,
            button = config.button_color,
            button_label = config.button_label_color,
            entry_background = config.entry_background_color,
            entry_text = config.entry_text_color,
        ));

        root.fullscreen();

        widgets.grid.attach(&timer_frame, 0, 0, 1, 1);
        widgets.grid.attach(&hint_menu_frame, 1, 0, 1, 1);
        widgets.grid.attach(&play_video_button, 0, 1, 1, 1);
        widgets.grid.attach(&play_audio_button, 1, 1, 1, 1);
        widgets.grid.attach(&hint_frame, 0, 2, 1, 1);
        widgets.grid.attach(&player_preview_frame, 1, 2, 1, 2);
        widgets.grid.attach(&misc_frame, 0, 3, 1, 1);

        ComponentParts { model, widgets }
    }
}
