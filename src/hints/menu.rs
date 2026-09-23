use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::utils::read_toml;

const MENU_MAX_HEIGHT: i32 = 200;

struct HintTable {
    name: String,
    hints: Vec<String>,
}

pub struct HintMenu {
    tables: Vec<HintTable>,
}

#[derive(Debug)]
pub enum HintMenuMsg {
    HintClicked(String),
    HintActivated(String),
}

#[derive(Debug)]
pub enum HintMenuOutput {
    HintSelected(String),
    HintActivated(String),
}

#[relm4::component(pub)]
impl SimpleComponent for HintMenu {
    type Init = ();

    type Input = HintMenuMsg;
    type Output = HintMenuOutput;

    view! {
        gtk::ScrolledWindow {
            set_hexpand: true,
            set_propagate_natural_height: true,
            set_max_content_height: MENU_MAX_HEIGHT,

            #[name = "list"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HintMenu {
            tables: load_hint_tables(),
        };

        let widgets = view_output!();

        for table in &model.tables {
            let expander = gtk::Expander::new(Some(&table.name));
            expander.set_expanded(true);

            let hints_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            for hint in &table.hints {
                let label = gtk::Label::new(Some(hint));
                label.set_halign(gtk::Align::Start);

                let click = gtk::GestureClick::new();
                let value = hint.clone();
                let input_sender = sender.input_sender().clone();
                click.connect_pressed(move |_, n_press, _, _| {
                    if n_press == 1 {
                        input_sender.emit(HintMenuMsg::HintClicked(value.clone()));
                    } else if n_press == 2 {
                        input_sender.emit(HintMenuMsg::HintActivated(value.clone()));
                    }
                });
                label.add_controller(click);

                hints_box.append(&label);
            }
            expander.set_child(Some(&hints_box));

            widgets.list.append(&expander);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            HintMenuMsg::HintClicked(value) => {
                let _ = sender.output(HintMenuOutput::HintSelected(value));
            }
            HintMenuMsg::HintActivated(value) => {
                let _ = sender.output(HintMenuOutput::HintActivated(value));
            }
        }
    }
}

/// Loads the hints from the hints file the CLI was given, if it was given one.
fn load_hint_tables() -> Vec<HintTable> {
    let Some(path) = super::path() else {
        return Vec::new();
    };

    read_toml(path)
        .iter()
        .filter_map(|(name, value)| {
            let table = value.as_table()?;

            let box_name = table.get("box_name").and_then(|v| v.as_str()).unwrap_or("");
            let box_contents = table
                .get("box_contents")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let hints = table
                .iter()
                .filter(|(key, _)| *key != "box_name" && *key != "box_contents")
                .filter_map(|(_, value)| value.as_str().map(str::to_string))
                .collect();

            Some(HintTable {
                name: format!("{box_name}({name}):{box_contents}"),
                hints,
            })
        })
        .collect()
}
