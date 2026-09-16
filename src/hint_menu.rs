use std::fs;

use gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

const HINTS_TOML_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/hints.toml");

struct HintTable {
    name: String,
    hints: Vec<String>,
}

pub struct HintMenu {
    tables: Vec<HintTable>,
}

#[relm4::component(pub)]
impl SimpleComponent for HintMenu {
    type Init = ();

    type Input = ();
    type Output = ();

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
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HintMenu {
            tables: load_hint_tables(),
        };

        let widgets = view_output!();

        for table in &model.tables {
            let expander = gtk::Expander::new(Some(&table.name));

            let hints_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            for hint in &table.hints {
                let label = gtk::Label::new(Some(hint));
                label.set_halign(gtk::Align::Start);
                label.set_wrap(true);
                hints_box.append(&label);
            }
            expander.set_child(Some(&hints_box));

            widgets.list.append(&expander);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
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

            let mut hints: Vec<(u32, String)> = table
                .iter()
                .filter_map(|(key, value)| {
                    let index = key.strip_prefix("hint")?.parse::<u32>().ok()?;
                    let text = value.as_str()?.to_string();
                    Some((index, text))
                })
                .collect();
            hints.sort_by_key(|(index, _)| *index);

            Some(HintTable {
                name: name.clone(),
                hints: hints.into_iter().map(|(_, text)| text).collect(),
            })
        })
        .collect()
}
