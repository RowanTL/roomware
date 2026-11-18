//! Holds the hint dropdown menus and the logic for reading hints in.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::{AdjustmentExt, BoxExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::utils::read_toml;

const MENU_MAX_HEIGHT: i32 = 200;
const SELECTED_CLASS: &str = "hint-selected";

struct HintTable {
    name: String,
    hints: Vec<String>,
}

pub struct HintMenu {
    expanders: Vec<gtk::Expander>,
    root: gtk::ScrolledWindow,
    list: gtk::Box,
    selected: Rc<RefCell<Option<gtk::Label>>>,
}

#[derive(Debug)]
pub enum HintMenuMsg {
    HintClicked(String),
    HintActivated(String),
    ExpandAll,
    /// Rebuilds the menu from the hints file.
    Reload,
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

            #[local_ref]
            list -> gtk::Box {
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
        let mut model = HintMenu {
            expanders: Vec::new(),
            root: root.clone(),
            list: gtk::Box::default(),
            selected: Rc::default(),
        };

        let list = &model.list;
        let widgets = view_output!();

        model.fill(&sender);

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
            HintMenuMsg::ExpandAll => {
                for expander in &self.expanders {
                    expander.set_expanded(true);
                }
                if let Some(label) = self.selected.take() {
                    label.remove_css_class(SELECTED_CLASS);
                }
                // scroll hint menu back to top
                self.root.vadjustment().set_value(0.0);
            }
            HintMenuMsg::Reload => {
                while let Some(child) = self.list.first_child() {
                    self.list.remove(&child);
                }
                self.expanders.clear();
                self.selected.take();
                self.fill(&sender);
                self.root.vadjustment().set_value(0.0);
            }
        }
    }
}

impl HintMenu {
    fn fill(&mut self, sender: &ComponentSender<Self>) {
        for table in load_hint_tables() {
            let expander = gtk::Expander::new(Some(&table.name));
            expander.set_expanded(true);
            self.expanders.push(expander.clone());

            let hints_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
            for hint in table.hints {
                let label = gtk::Label::new(Some(&format!("• {hint}")));
                label.set_halign(gtk::Align::Start);

                let click = gtk::GestureClick::new();
                let input_sender = sender.input_sender().clone();
                let selected = self.selected.clone();
                let this = label.clone();
                click.connect_pressed(move |_, n_press, _, _| {
                    if n_press == 1 {
                        if let Some(previous) = selected.replace(Some(this.clone())) {
                            previous.remove_css_class(SELECTED_CLASS);
                        }
                        this.add_css_class(SELECTED_CLASS);
                        input_sender.emit(HintMenuMsg::HintClicked(hint.clone()));
                    } else if n_press == 2 {
                        input_sender.emit(HintMenuMsg::HintActivated(hint.clone()));
                    }
                });
                label.add_controller(click);

                hints_box.append(&label);
            }
            expander.set_child(Some(&hints_box));

            self.list.append(&expander);
        }
    }
}

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
                name: format!("{box_name}({name}): {box_contents}"),
                hints,
            })
        })
        .collect()
}
