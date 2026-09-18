use gtk::glib;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

// In seconds, 3600 seconds = 1 hour
pub(crate) const DEFAULT_TIME_LIMIT: u32 = 2;

pub struct Timer {
    remaining: u32,
    running: bool,
    outcome: Outcome,
    source: Option<glib::SourceId>,
}

/// How the game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Undecided,
    Won,
    Lost,
}

#[derive(Debug)]
pub enum TimerMsg {
    Toggle,
    Reset,
    Win,
    Tick,
}

#[derive(Debug)]
pub enum TimerOutput {
    TimeChanged(u32),
    OutcomeChanged(Outcome),
}

#[relm4::component(pub)]
impl SimpleComponent for Timer {
    type Init = ();

    type Input = TimerMsg;
    type Output = TimerOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            set_margin_all: 5,

            gtk::Button {
                #[watch]
                set_label: if model.running { "Stop" } else { "Start" },
                connect_clicked => TimerMsg::Toggle,
            },

            gtk::Button::with_label("Reset") {
                connect_clicked => TimerMsg::Reset,
            },

            gtk::Button::with_label("Win") {
                connect_clicked => TimerMsg::Win,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Timer {
            remaining: DEFAULT_TIME_LIMIT,
            running: false,
            outcome: Outcome::Undecided,
            source: None,
        };

        let widgets = view_output!();

        let _ = sender.output(TimerOutput::TimeChanged(model.remaining));
        let _ = sender.output(TimerOutput::OutcomeChanged(model.outcome));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TimerMsg::Toggle => {
                if self.running {
                    self.stop();
                } else {
                    self.start(&sender);
                }
            }
            TimerMsg::Reset => {
                self.stop();
                self.remaining = DEFAULT_TIME_LIMIT;
                self.outcome = Outcome::Undecided;
            }
            TimerMsg::Win => {
                self.stop();
                self.outcome = Outcome::Won;
            }
            TimerMsg::Tick => {
                self.remaining = self.remaining.saturating_sub(1);
                if self.remaining == 0 {
                    self.stop();
                    self.outcome = Outcome::Lost;
                }
            }
        }

        let _ = sender.output(TimerOutput::TimeChanged(self.remaining));
        let _ = sender.output(TimerOutput::OutcomeChanged(self.outcome));
    }
}

impl Timer {
    fn start(&mut self, sender: &ComponentSender<Self>) {
        if self.remaining == 0 || self.outcome != Outcome::Undecided {
            return;
        }
        self.running = true;

        let input_sender = sender.input_sender().clone();
        let source = glib::timeout_add_seconds_local(1, move || {
            input_sender.emit(TimerMsg::Tick);
            glib::ControlFlow::Continue
        });
        self.source = Some(source);
    }

    fn stop(&mut self) {
        self.running = false;
        if let Some(source) = self.source.take() {
            source.remove();
        }
    }
}

/// Takes a time in seconds and makes a nice output string
/// from it.
pub(crate) fn format_time(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
