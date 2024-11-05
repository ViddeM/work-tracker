use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, clap::ValueEnum)]
pub enum WorkEntryStatus {
    Created,
    Completed,
}

impl WorkEntryStatus {
    pub fn get_icon(&self) -> String {
        match self {
            WorkEntryStatus::Created => String::new(),
            WorkEntryStatus::Completed => format!("{}", "✔".green()),
        }
    }

    pub fn to_colored_string(&self) -> ColoredString {
        match self {
            WorkEntryStatus::Created => "Created".bright_blue(),
            WorkEntryStatus::Completed => "Completed".green(),
        }
    }
}
