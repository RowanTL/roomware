use relm4::RelmApp;
use roomware::master::GameMasterWindow;

fn main() {
    let app = RelmApp::new("EscapeOnMain.RoomWare.Monitor");
    app.run::<GameMasterWindow>(());
}
