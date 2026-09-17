use std::fs;

use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

const HINTS_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/hints.toml");

// Recognised `hints.toml` key prefixes, each followed by a number. Text hints
// are listed before pictures within a step; both sort by their number.
const KEY_PREFIXES: [&str; 2] = ["hint", "pic"];

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

/// Split a key such as `hint2` or `pic1` into the sort order of its prefix and
/// its number. Keys that match no prefix are skipped entirely.
fn parse_key(key: &str) -> Option<(usize, u32)> {
    KEY_PREFIXES.iter().enumerate().find_map(|(group, prefix)| {
        let index = key.strip_prefix(prefix)?.parse::<u32>().ok()?;
        Some((group, index))
    })
}

fn load_hint_tables() -> Vec<HintTable> {
    let Ok(contents) = fs::read_to_string(HINTS_TOML_PATH) else {
        return Vec::new();
    };

    let Ok(root) = contents.parse::<toml::Table>() else {
        return Vec::new();
    };

    root.iter()
        .filter_map(|(name, value)| {
            let table = value.as_table()?;

            let mut hints: Vec<((usize, u32), Hint)> = table
                .iter()
                .filter_map(|(key, value)| {
                    let order = parse_key(key)?;
                    let value = value.as_str()?.to_string();
                    Some((
                        order,
                        Hint {
                            key: key.clone(),
                            value,
                        },
                    ))
                })
                .collect();
            hints.sort_by_key(|(order, _)| *order);

            Some(HintTable {
                name: name.clone(),
                hints: hints.into_iter().map(|(_, hint)| hint).collect(),
            })
        })
        .collect()
}
