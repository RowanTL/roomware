use gtk::glib;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt};
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

// In seconds, 3600 seconds = 1 hour
pub(crate) const DEFAULT_TIME_LIMIT: u32 = 3600;

pub struct Timer {
    remaining: u32,
    running: bool,
    source: Option<glib::SourceId>,
}

#[derive(Debug)]
pub enum TimerMsg {
    Toggle,
    Reset,
    Tick,
}

#[derive(Debug)]
pub enum TimerOutput {
    TimeChanged(u32),
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

            gtk::Label {
                #[watch]
                set_label: &format_time(model.remaining),
                set_margin_all: 5,
            },

            gtk::Button {
                #[watch]
                set_label: if model.running { "Stop" } else { "Start" },
                connect_clicked => TimerMsg::Toggle,
            },

            gtk::Button::with_label("Reset") {
                connect_clicked => TimerMsg::Reset,
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
            source: None,
        };

        let widgets = view_output!();

        let _ = sender.output(TimerOutput::TimeChanged(model.remaining));

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
            }
            TimerMsg::Tick => {
                self.remaining = self.remaining.saturating_sub(1);
                if self.remaining == 0 {
                    self.stop();
                }
            }
        }

        let _ = sender.output(TimerOutput::TimeChanged(self.remaining));
    }
}

impl Timer {
    fn start(&mut self, sender: &ComponentSender<Self>) {
        if self.remaining == 0 {
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

pub(crate) fn format_time(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
