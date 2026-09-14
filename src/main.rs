use gtk::prelude::GtkWindowExt;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    SimpleComponent, gtk,
};
use roomware::hints::Hints;

struct MainWindow {
    hints: Controller<Hints>,
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

            #[local_ref]
            hints_box -> gtk::Box {}
        }
    }

    // Initialize the UI.
    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let hints = Hints::builder().launch(0).detach();

        let model = MainWindow { hints };

        let hints_box = model.hints.widget();

        // Insert the macro code generation here
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor");
    app.run::<MainWindow>(());
}
