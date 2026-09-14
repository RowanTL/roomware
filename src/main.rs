use gtk::prelude::{GridExt, GtkWindowExt};
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
    // dead_code here to keep Controller alive
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

        let model = MainWindow {
            hints,
            timer,
            player,
        };

        // Insert the macro code generation here
        let widgets = view_output!();

        widgets.grid.attach(model.timer.widget(), 0, 0, 1, 1);
        widgets.grid.attach(model.hints.widget(), 1, 0, 1, 1);

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor");
    app.run::<MainWindow>(());
}
