use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::utils::read_toml;

struct Hint {
    key: String,
    value: String,
}

struct HintTable {
    name: String,
    hints: Vec<Hint>,
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
            set_vexpand: true,
            set_hscrollbar_policy: gtk::PolicyType::Never,

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
                let label = gtk::Label::new(Some(&hint.key));
                label.set_halign(gtk::Align::Start);
                label.set_wrap(true);

                let click = gtk::GestureClick::new();
                let value = hint.value.clone();
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

            let hints = table
                .iter()
                .filter_map(|(key, value)| {
                    Some(Hint {
                        key: key.clone(),
                        value: value.as_str()?.to_string(),
                    })
                })
                .collect();

            Some(HintTable {
                name: name.clone(),
                hints,
            })
        })
        .collect()
}
