//! Records the outcome of each game to the csv file at `master.log.path`. Runs as a
//! worker so file writes stay off the UI thread.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use relm4::{ComponentSender, Worker};

use crate::config::config;
use crate::timer::Outcome;

const HEADER: &str =
    "room,outcome,time_left,initial_time,hints_left,initial_hints,start_time,end_time";
const MIN_GAME_LENGTH: u32 = 15;

pub struct Stats {
    log: Option<GameLog>,
    hints_remaining: u8,
    force_loss: bool,
    // The row written for the current game, if any, holding the real outcome so
    // toggling force loss off can restore a win
    recorded: Option<Record>,
}

#[derive(Debug)]
pub enum StatsMsg {
    HintsChanged(u8),
    GameOver {
        outcome: Outcome,
        remaining: u32,
        elapsed: u32,
        start_time: String,
        end_time: String,
    },
    ForceLoss(bool),
    Reset,
}

impl Worker for Stats {
    type Init = ();

    type Input = StatsMsg;
    type Output = ();

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        let log = config()
            .master_log_path
            .as_deref()
            .and_then(|path| match GameLog::open(path) {
                Ok(log) => Some(log),
                Err(error) => {
                    eprintln!("roomware: could not open {}: {error}", path.display());
                    None
                }
            });

        Self {
            log,
            hints_remaining: config().hints_default_amount,
            force_loss: false,
            recorded: None,
        }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            StatsMsg::HintsChanged(remaining) => {
                self.hints_remaining = remaining;
            }
            StatsMsg::GameOver {
                outcome,
                remaining,
                elapsed,
                start_time,
                end_time,
            } => {
                if elapsed < MIN_GAME_LENGTH {
                    return;
                }

                let record = Record {
                    room: &config().master_room_name,
                    won: outcome == Outcome::Won,
                    time_left: remaining,
                    initial_time: config().master_time,
                    hints_left: self.hints_remaining,
                    initial_hints: config().hints_default_amount,
                    start_time,
                    end_time,
                };
                // The outcome can change mid-game (e.g. a loss undone by +1 minute),
                // so a game only ever owns one row
                let replace = self.recorded.is_some();
                self.write(&record, replace);
                self.recorded = Some(record);
            }
            StatsMsg::ForceLoss(force_loss) => {
                self.force_loss = force_loss;

                // A loss reads the same either way, so only a recorded win needs rewriting
                if let Some(record) = self.recorded.clone().filter(|record| record.won) {
                    self.write(&record, true);
                }
            }
            StatsMsg::Reset => {
                self.force_loss = false;
                self.recorded = None;
            }
        }
    }
}

impl Stats {
    fn write(&self, record: &Record, replace: bool) {
        let Some(log) = &self.log else {
            return;
        };

        let mut record = record.clone();
        record.won = record.won && !self.force_loss;
        let result = if replace {
            log.replace_last(&record)
        } else {
            log.append(&record)
        };

        if let Err(error) = result {
            eprintln!("roomware: could not write {}: {error}", log.path.display());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    room: &'static str,
    won: bool,
    time_left: u32,
    initial_time: u32,
    hints_left: u8,
    initial_hints: u8,
    start_time: String,
    end_time: String,
}

impl Record {
    fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{}",
            csv_field(self.room),
            if self.won { "win" } else { "loss" },
            self.time_left,
            self.initial_time,
            self.hints_left,
            self.initial_hints,
            csv_field(&self.start_time),
            csv_field(&self.end_time),
        )
    }
}

/// Quotes text that would otherwise break the row apart, doubling any quotes inside it.
fn csv_field(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

struct GameLog {
    path: PathBuf,
}

impl GameLog {
    /// Creates the file if it doesn't exist and gives an empty one a header row.
    fn open(path: &Path) -> io::Result<Self> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        if file.metadata()?.len() == 0 {
            writeln!(file, "{HEADER}")?;
        }

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    fn append(&self, record: &Record) -> io::Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(file, "{}", record.to_csv())
    }

    fn replace_last(&self, record: &Record) -> io::Result<()> {
        let contents = fs::read_to_string(&self.path)?;
        let contents = contents.trim_end();
        let kept = contents.rfind('\n').map_or("", |end| &contents[..=end]);

        fs::write(&self.path, format!("{kept}{}\n", record.to_csv()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> Record {
        Record {
            room: "Museum",
            won: true,
            time_left: 600,
            initial_time: 3600,
            hints_left: 2,
            initial_hints: 5,
            start_time: "2024-01-01T10:00:00+0000".to_string(),
            end_time: "2024-01-01T11:00:00+0000".to_string(),
        }
    }

    fn temp_csv(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("roomware-{name}-{}.csv", std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn open_creates_missing_file() {
        let path = temp_csv("create");
        assert!(!path.exists());

        GameLog::open(&path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), format!("{HEADER}\n"));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn replace_last_rewrites_only_the_last_row() {
        let path = temp_csv("replace");
        let log = GameLog::open(&path).unwrap();

        log.append(&record()).unwrap();
        log.append(&record()).unwrap();
        log.replace_last(&Record {
            won: false,
            ..record()
        })
        .unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!(
                "{HEADER}\n\
                 Museum,win,600,3600,2,5,2024-01-01T10:00:00+0000,2024-01-01T11:00:00+0000\n\
                 Museum,loss,600,3600,2,5,2024-01-01T10:00:00+0000,2024-01-01T11:00:00+0000\n"
            )
        );
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn room_names_with_commas_or_quotes_are_quoted() {
        let record = Record {
            room: "Bob's \"Big\", Room",
            ..record()
        };

        assert_eq!(
            record.to_csv(),
            "\"Bob's \"\"Big\"\", Room\",win,600,3600,2,5,\
             2024-01-01T10:00:00+0000,2024-01-01T11:00:00+0000"
        );
    }
}
