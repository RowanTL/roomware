//! Holds timer information. Declares the timer control buttons as well. Also, this
//! is where the countdown logic is stored.

use gtk::glib;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

use crate::config::config;

// How much the +1 / -1 minute buttons move the clock, in seconds.
const ADJUSTMENT: u32 = 60;

pub struct Timer {
    remaining: u32,
    running: bool,
    outcome: Outcome,
    // Seconds the clock has actually run since the last reset
    elapsed: u32,
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
    AddMinute,
    SubtractMinute,
    Tick,
}

#[derive(Debug)]
pub enum TimerOutput {
    TimeChanged(u32),
    OutcomeChanged(Outcome),
    GameOver {
        outcome: Outcome,
        remaining: u32,
        elapsed: u32,
    },
    Reset,
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

            gtk::Button::with_label("+1 Minute") {
                connect_clicked => TimerMsg::AddMinute,
            },

            gtk::Button::with_label("-1 Minute") {
                connect_clicked => TimerMsg::SubtractMinute,
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Timer {
            remaining: config().master_time,
            running: false,
            outcome: Outcome::Undecided,
            elapsed: 0,
            source: None,
        };

        let widgets = view_output!();

        let _ = sender.output(TimerOutput::TimeChanged(model.remaining));
        let _ = sender.output(TimerOutput::OutcomeChanged(model.outcome));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        let previous = self.outcome;

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
                self.remaining = config().master_time;
                self.outcome = Outcome::Undecided;
                self.elapsed = 0;
                let _ = sender.output(TimerOutput::Reset);
            }
            TimerMsg::Win => {
                self.stop();
                self.outcome = Outcome::Won;
            }
            TimerMsg::AddMinute => {
                self.remaining = self.remaining.saturating_add(ADJUSTMENT);
                // There is time on the clock again, so the game is back to undecided
                if self.outcome == Outcome::Lost {
                    self.outcome = Outcome::Undecided;
                }
            }
            TimerMsg::SubtractMinute => {
                self.remaining = self.remaining.saturating_sub(ADJUSTMENT);
                if self.remaining == 0 {
                    self.time_up();
                }
            }
            TimerMsg::Tick => {
                self.remaining = self.remaining.saturating_sub(1);
                self.elapsed = self.elapsed.saturating_add(1);
                if self.remaining == 0 {
                    self.time_up();
                }
            }
        }

        let _ = sender.output(TimerOutput::TimeChanged(self.remaining));
        let _ = sender.output(TimerOutput::OutcomeChanged(self.outcome));

        if self.outcome != previous && self.outcome != Outcome::Undecided {
            let _ = sender.output(TimerOutput::GameOver {
                outcome: self.outcome,
                remaining: self.remaining,
                elapsed: self.elapsed,
            });
        }
    }
}

impl Timer {
    fn start(&mut self, sender: &ComponentSender<Self>) {
        if self.remaining == 0 || self.outcome != Outcome::Undecided {
            return;
        }
        self.running = true;

        let input_sender = sender.input_sender().clone();
        // Every 1 second, this function calls whatever is inside of the closure
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

    fn time_up(&mut self) {
        self.stop();
        self.outcome = Outcome::Lost;
    }
}

/// Output format as {minutes:02}:{seconds:02}
pub(crate) fn format_time(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
