//! This file holds the text entry box and the show/clear hint buttons.

use std::collections::VecDeque;

use gtk::gdk::Key;
use gtk::glib::Propagation;
use gtk::prelude::{
    BoxExt, ButtonExt, Cast, EditableExt, EntryExt, EventControllerExt, GtkWindowExt,
    OrientableExt, WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

use crate::hints;
use crate::utils::load_image;

const HISTORY_LEN: usize = 5;

/// Sent hints, newest first. Recalled entries are copied into the entry, so edits never touch them.
#[derive(Default)]
struct History {
    entries: VecDeque<String>,
    pos: Option<usize>,
    draft: String,
}

impl History {
    fn record(&mut self, text: &str) {
        self.reset();
        if text.trim().is_empty() || self.entries.front().is_some_and(|e| e == text) {
            return;
        }
        self.entries.push_front(text.to_owned());
        self.entries.truncate(HISTORY_LEN);
    }

    fn reset(&mut self) {
        self.pos = None;
    }

    fn older(&mut self, current: &str) -> Option<&str> {
        let next = self.pos.map_or(0, |p| p + 1);
        if next >= self.entries.len() {
            return None;
        }
        if self.pos.is_none() {
            self.draft = current.to_owned();
        }
        self.pos = Some(next);
        Some(&self.entries[next])
    }

    fn newer(&mut self) -> Option<&str> {
        match self.pos? {
            0 => {
                self.pos = None;
                Some(&self.draft)
            }
            p => {
                self.pos = Some(p - 1);
                Some(&self.entries[p - 1])
            }
        }
    }
}

pub struct HintControl {
    entry: Option<gtk::Entry>,
    history: History,
}

#[derive(Debug)]
pub enum HintControlMsg {
    Show,
    PreviewImage,
    Submit,
    Clear,
    SetText(String),
    ShowValue(String),
    HistoryOlder,
    HistoryNewer,
}

#[derive(Debug)]
pub enum HintControlOutput {
    ShowHint(String),
}

#[relm4::component(pub)]
impl SimpleComponent for HintControl {
    type Init = ();

    type Input = HintControlMsg;
    type Output = HintControlOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 5,

            #[name = "entry"]
            gtk::Entry {
                set_hexpand: true,
                connect_activate => HintControlMsg::Submit,

                // Capture so the entry's own Up/Down focus navigation never sees the key.
                add_controller = gtk::EventControllerKey {
                    set_propagation_phase: gtk::PropagationPhase::Capture,
                    connect_key_pressed[sender] => move |_, key, _, _| match key {
                        Key::Up | Key::KP_Up => {
                            sender.input(HintControlMsg::HistoryOlder);
                            Propagation::Stop
                        }
                        Key::Down | Key::KP_Down => {
                            sender.input(HintControlMsg::HistoryNewer);
                            Propagation::Stop
                        }
                        _ => Propagation::Proceed,
                    },
                },
            },

            gtk::Button::with_label("Preview Image") {
                connect_clicked => HintControlMsg::PreviewImage,
            },

            gtk::Button::with_label("Show Hint") {
                connect_clicked => HintControlMsg::Show,
            },

            gtk::Button::with_label("Clear Hint") {
                connect_clicked => HintControlMsg::Clear,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = HintControl {
            entry: None,
            history: History::default(),
        };

        let widgets = view_output!();

        model.entry = Some(widgets.entry.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        let Some(entry) = &self.entry else {
            return;
        };

        match msg {
            HintControlMsg::Show => {
                let hint = entry.text().to_string();
                self.history.record(&hint);
                let _ = sender.output(HintControlOutput::ShowHint(hint));
                entry.set_text("");
            }
            HintControlMsg::PreviewImage => {
                let text = entry.text();
                if text.trim().is_empty() {
                    return;
                }

                let window = entry
                    .root()
                    .and_then(|root| root.downcast::<gtk::Window>().ok());

                let Some(texture) = load_image(hints::base(), &text) else {
                    gtk::AlertDialog::builder()
                        .message("this is text stupid head")
                        .build()
                        .show(window.as_ref());
                    return;
                };

                let preview = gtk::Window::builder()
                    .title("Image preview")
                    .default_width(640)
                    .default_height(480)
                    .child(&gtk::Picture::for_paintable(&texture))
                    .build();
                preview.set_transient_for(window.as_ref());
                preview.present();
            }
            // Happens whenever user clicks enter on their keyboard.
            // Apparently this functionality is baked into gtk's Entry widget
            HintControlMsg::Submit => {
                let hint = entry.text().to_string();

                // If hint box is empty, do nothing
                if hint.trim().is_empty() {
                    return;
                }

                self.history.record(&hint);
                let _ = sender.output(HintControlOutput::ShowHint(hint));
                entry.set_text("");
            }
            HintControlMsg::Clear => {
                self.history.reset();
                let _ = sender.output(HintControlOutput::ShowHint(String::new()));
                entry.set_text("");
            }
            HintControlMsg::SetText(text) => {
                self.history.reset();
                entry.set_text(&text);
            }
            HintControlMsg::ShowValue(text) => {
                self.history.record(&text);
                let _ = sender.output(HintControlOutput::ShowHint(text));
                entry.set_text("");
            }
            HintControlMsg::HistoryOlder => {
                if let Some(text) = self.history.older(&entry.text()) {
                    entry.set_text(text);
                    entry.set_position(-1);
                }
            }
            HintControlMsg::HistoryNewer => {
                if let Some(text) = self.history.newer() {
                    entry.set_text(text);
                    entry.set_position(-1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(items: &[&str]) -> History {
        let mut h = History::default();
        for item in items {
            h.record(item);
        }
        h
    }

    #[test]
    fn keeps_only_last_five() {
        let mut h = history(&["1", "2", "3", "4", "5", "6"]);
        let recalled: Vec<_> = std::iter::from_fn(|| h.older("").map(str::to_owned)).collect();
        assert_eq!(recalled, ["6", "5", "4", "3", "2"]);
    }

    #[test]
    fn down_past_newest_restores_draft() {
        let mut h = history(&["a", "b"]);
        assert_eq!(h.older("typing"), Some("b"));
        assert_eq!(h.older("b edited"), Some("a"));
        assert_eq!(h.newer(), Some("b"));
        assert_eq!(h.newer(), Some("typing"));
        assert_eq!(h.newer(), None);
    }

    #[test]
    fn editing_recalled_entry_keeps_original() {
        let mut h = history(&["a"]);
        h.older("");
        h.record("a changed");
        assert_eq!(h.older(""), Some("a changed"));
        assert_eq!(h.older(""), Some("a"));
    }

    #[test]
    fn skips_blank_and_repeated() {
        let mut h = history(&["a", "a", "  "]);
        assert_eq!(h.older(""), Some("a"));
        assert_eq!(h.older(""), None);
    }
}
