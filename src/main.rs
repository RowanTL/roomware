use gtk::prelude::{ButtonExt, EditableExt, GridExt, GtkWindowExt};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent, gtk,
};
use roomware::hints::{DEFAULT_HINT_AMT, Hints};
use roomware::player::{Player, PlayerMsg};
use roomware::timer::{Timer, TimerOutput};

struct MainWindow {
    hints: Controller<Hints>,
    timer: Controller<Timer>,
    // dead_code here to keep player Controller alive
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
            set_default_width: 300,
            set_default_height: 100,

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
        let hints = Hints::builder().launch(DEFAULT_HINT_AMT).detach();
        let player = Player::builder().launch(()).detach();
        let timer = Timer::builder()
            .launch(())
            .forward(player.sender(), |output| match output {
                TimerOutput::TimeChanged(remaining) => PlayerMsg::UpdateTime(remaining),
            });

        player.widget().present();

        let play_video_button = gtk::Button::with_label("Play Video");
        let player_sender = player.sender().clone();
        play_video_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::PlayVideo);
        });

        let hint_entry = gtk::Entry::new();
        let show_message_button = gtk::Button::with_label("Show Hint");
        let clear_message_button = gtk::Button::with_label("Clear Hint");

        let player_sender = player.sender().clone();
        let entry_for_message = hint_entry.clone();
        show_message_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::ShowHint(entry_for_message.text().to_string()));
            entry_for_message.set_text("");
        });

        let player_sender = player.sender().clone();
        let entry_for_message = hint_entry.clone();
        clear_message_button.connect_clicked(move |_| {
            player_sender.emit(PlayerMsg::ShowHint(String::from("")));
            entry_for_message.set_text("");
        });

        let model = MainWindow {
            hints,
            timer,
            player,
        };

        // Insert the macro code generation here
        let widgets = view_output!();

        widgets.grid.attach(model.timer.widget(), 0, 0, 1, 1);
        widgets.grid.attach(model.hints.widget(), 1, 0, 1, 1);
        widgets.grid.attach(&play_video_button, 0, 1, 2, 1);
        widgets.grid.attach(&hint_entry, 0, 2, 1, 1);
        widgets.grid.attach(&show_message_button, 1, 2, 1, 1);
        widgets.grid.attach(&clear_message_button, 0, 3, 2, 1);

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor");
    app.run::<MainWindow>(());
}
