use gtk::prelude::{BoxExt, ButtonExt, FrameExt, GridExt, GtkWindowExt, WidgetExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent, gtk,
};
use roomware::hints::control::{HintControl, HintControlMsg, HintControlOutput};
use roomware::hints::menu::{HintMenu, HintMenuOutput};
use roomware::hints::{DEFAULT_HINT_AMT, Hints, HintsOutput};
use roomware::player::preview::{PlayerPreview, PlayerPreviewMsg};
use roomware::player::{Player, PlayerMsg, PlayerOutput};
use roomware::timer::{Timer, TimerOutput};
use roomware::utils::apply_css_style;

struct MainWindow {
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
}

#[relm4::component]
impl SimpleComponent for MainWindow {
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
            .launch(DEFAULT_HINT_AMT)
            .forward(player.sender(), |output| match output {
                HintsOutput::CounterChanged(remaining) => {
                    PlayerMsg::UpdateHintsRemaining(remaining)
                }
            });
        let timer = Timer::builder()
            .launch(())
            .forward(player.sender(), |output| match output {
                TimerOutput::TimeChanged(remaining) => PlayerMsg::UpdateTime(remaining),
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

        player.widget().present();

        let play_video_button = gtk::Button::with_label("Play Intro");
        let player_sender = player.sender().clone();
        play_video_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::PlayVideo);
        });

        let timer_frame = gtk::Frame::new(Some("Timer Controls"));
        timer_frame.set_child(Some(timer.widget()));

        let hint_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
        hint_box.append(hints.widget());
        hint_box.append(hint_control.widget());

        let hint_frame = gtk::Frame::new(Some("Hint Controls"));
        hint_frame.set_child(Some(&hint_box));

        let player_preview_frame = gtk::Frame::new(Some("Player Screen Preview"));
        player_preview_frame.set_child(Some(player_preview.widget()));
        player_preview_frame.set_hexpand(true);

        let hint_menu_frame = gtk::Frame::new(Some("Hint Selection"));
        hint_menu_frame.set_child(Some(hint_menu.widget()));
        hint_menu_frame.set_hexpand(false);

        let model = MainWindow {
            hints,
            timer,
            hint_control,
            player_preview,
            hint_menu,
            player,
        };

        // Insert the macro code generation here
        let widgets = view_output!();

        apply_css_style(
            "window.main-window { background-color: #333333; } \
             window.main-window frame { border: 2px solid #D3D3D3; padding: 6px; } \
             window.main-window frame label { color: #D3D3D3; } \
             window.main-window frame > label { font-weight: bold; } \
             window.main-window expander title { color: #D3D3D3; } \
             window.main-window expander title arrow { color: #D3D3D3; } \
             window.main-window button { background-color: #D3D3D3; color: #1A1A1A; } \
             window.main-window frame button label { color: #1A1A1A; } \
             window.main-window entry { background-color: #D3D3D3; color: #1A1A1A; }",
        );

        root.fullscreen();

        widgets.grid.attach(&timer_frame, 0, 0, 1, 1);
        widgets.grid.attach(&hint_frame, 1, 0, 1, 1);
        widgets.grid.attach(&play_video_button, 0, 1, 2, 1);
        widgets.grid.attach(&hint_menu_frame, 0, 2, 1, 1);
        widgets.grid.attach(&player_preview_frame, 1, 2, 1, 1);

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor");
    app.run::<MainWindow>(());
}
