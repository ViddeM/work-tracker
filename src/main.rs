use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::{ColoredString, Colorize};
use eyre::{Context, ContextCompat, OptionExt};
use home::home_dir;
use serde::{Deserialize, Serialize};

const MAX_NAME_LENGTH: usize = 28;

/// Simple program to keep track of work items.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Option<WorkAction>,
}

#[derive(Subcommand, Debug)]
enum WorkAction {
    /// Add a new work action entry.
    Add {
        /// What name the action should have (max 28 chars).
        name: String,
        /// Optiona description of the action.
        description: Option<String>,
    },
    /// Edit a work action entry.
    Edit {
        /// The ID of the entry to edit.
        id: usize,

        /// The new description of the entry.
        description: String,
    },
    /// List all unfinished work actions.
    List {
        /// Show all entries, not just completed ones.
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },
    /// Show detailed info for an entry.
    Show {
        /// The id of the entry to show.
        id: Option<usize>,
    },
    /// Removes the entry with the provided ID.
    Remove { id: usize },
    /// Marks the entry with the provided ID as completed.
    Complete { id: usize },
    /// Puts the task with the provided ID at the top of the list.
    Prio { id: usize },
}

#[derive(Serialize, Deserialize)]
struct WorkDataFile {
    version: FileVersion,
    entries: Vec<WorkEntry>,
}

impl WorkDataFile {
    pub fn new() -> Self {
        Self {
            version: FileVersion::Initial,
            entries: vec![],
        }
    }

    pub fn add_entry(&mut self, name: String, description: Option<String>) {
        let highest_num = self
            .entries
            .iter()
            .map(|e| &e.id)
            .max()
            .map(|v| v + 1)
            .unwrap_or(0);
        let new_entry = WorkEntry::new(highest_num, name, description);

        self.entries.push(new_entry);
    }

    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let serialized = ron::to_string(&self).wrap_err("Failed to serialize wd_file")?;

        let mut file = File::create(path).wrap_err("Failed to open config file to save changes")?;
        file.write_all(serialized.as_bytes())
            .wrap_err("Failed to write changes to file")?;

        Ok(())
    }

    fn get_index_for_id(&self, id: usize) -> eyre::Result<usize> {
        let (current_index, _) = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, entry)| entry.id == id)
            .wrap_err("No entry with the provided ID")?;

        Ok(current_index)
    }

    fn get_entry(&self, id: usize) -> eyre::Result<&WorkEntry> {
        Ok(self
            .entries
            .iter()
            .find(|entry| entry.id == id)
            .wrap_err("Failed to find entry with the provided ID")?)
    }

    fn get_entry_or_first(&self, id: Option<usize>) -> eyre::Result<Option<&WorkEntry>> {
        if let Some(id) = id {
            return self.get_entry(id).map(|e| Some(e));
        }

        return Ok(self.entries.iter().filter(|e| !e.is_completed()).last());
    }

    fn get_entry_mut(&mut self, id: usize) -> eyre::Result<&mut WorkEntry> {
        Ok(self
            .entries
            .iter_mut()
            .find(|entry| entry.id == id)
            .wrap_err("Failed to find entry with the provided ID")?)
    }
}

#[derive(Serialize, Deserialize)]
enum FileVersion {
    Initial,
}

#[derive(Serialize, Deserialize)]
struct WorkEntry {
    id: usize, // Just an incremental integer.
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    status: WorkEntryStatus,
}

impl WorkEntry {
    fn new(id: usize, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            status: WorkEntryStatus::Created,
        }
    }

    fn complete(&mut self) {
        self.modified_at = Utc::now();
        self.status = WorkEntryStatus::Completed;
    }

    fn to_printable_row(&self) -> String {
        format!(
            " {} {} {} {}",
            self.id,
            "->>".green(),
            self.name.bright_cyan(),
            self.status.get_icon(),
        )
    }

    fn is_completed(&self) -> bool {
        self.status == WorkEntryStatus::Completed
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
enum WorkEntryStatus {
    Created,
    Completed,
}

impl WorkEntryStatus {
    fn get_icon<'a>(&self) -> String {
        match self {
            WorkEntryStatus::Created => String::new(),
            WorkEntryStatus::Completed => format!("{}", "✔".green()),
        }
    }

    fn to_colored_string(&self) -> ColoredString {
        match self {
            WorkEntryStatus::Created => "Created".bright_blue(),
            WorkEntryStatus::Completed => todo!(),
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let config_path = home_dir()
        .ok_or_eyre("Failed to read home directory")?
        .join(".config/work-tracker.ron");

    let mut wd_file =
        get_or_create_file_file(&config_path).wrap_err("Failed to get or create config file")?;

    match args.action {
        None => {
            let latest = wd_file.entries.iter().filter(|e| !e.is_completed()).last();

            if let Some(l) = latest {
                println!("{}", l.to_printable_row());
            } else {
                println!("No active tasks, great job!");
            }
        }
        Some(WorkAction::Add { name, description }) => {
            if name.chars().count() > MAX_NAME_LENGTH {
                eyre::bail!("Name can have at most {MAX_NAME_LENGTH} chars");
            }

            wd_file.add_entry(name, description);

            wd_file.save(&config_path).wrap_err("Failed to save file")?;
        }
        Some(WorkAction::List { all }) => {
            let entries = wd_file.entries;
            for entry in entries.iter().rev() {
                if !all && entry.is_completed() {
                    continue;
                }

                println!("{}", entry.to_printable_row());
            }
        }
        Some(WorkAction::Remove { id }) => {
            let index = wd_file.get_index_for_id(id)?;
            wd_file.entries.remove(index);
            wd_file.save(&config_path).wrap_err("Failed to save file")?;
        }
        Some(WorkAction::Complete { id }) => {
            let entry = wd_file.get_entry_mut(id)?;
            eyre::ensure!(
                !entry.is_completed(),
                "Entry is already marked as completed"
            );
            entry.complete();
            wd_file
                .save(&config_path)
                .wrap_err("Failed to save changes")?;
        }
        Some(WorkAction::Prio { id }) => {
            let index = wd_file.get_index_for_id(id)?;

            let mut entry = wd_file.entries.remove(index);
            entry.modified_at = Utc::now();
            wd_file.entries.push(entry);

            wd_file
                .save(&config_path)
                .wrap_err("Failed to save changes")?;
        }
        Some(WorkAction::Edit { id, description }) => {
            let entry = wd_file.get_entry_mut(id)?;
            entry.description = Some(description);
        }
        Some(WorkAction::Show { id }) => {
            let Some(WorkEntry {
                id,
                name,
                description,
                created_at,
                modified_at,
                status,
            }) = wd_file.get_entry_or_first(id)?
            else {
                println!("{}", "No unfinished tasks!".bright_green());
                return Ok(());
            };

            let div = "::".truecolor(175, 175, 175);
            println!(
                "{} {div} {} {div} {} {div} {} {div} {created_at:?} / {modified_at:?}",
                id.to_string().bright_blue(),
                name.bright_green(),
                description
                    .as_ref()
                    .unwrap_or(&"<No description>".to_string())
                    .yellow(),
                status.to_colored_string()
            )
        }
    };

    Ok(())
}

fn get_or_create_file_file(path: &Path) -> eyre::Result<WorkDataFile> {
    if !path.exists() {
        let wd_file = WorkDataFile::new();
        let mut file = File::create_new(path).wrap_err("Failed to create config file")?;
        let serialized =
            ron::to_string(&wd_file).wrap_err("Failed to serialize initial wd_file")?;
        file.write_all(serialized.as_bytes())
            .wrap_err("Failed to write initial object to config file")?;

        return Ok(wd_file);
    }

    let mut file = File::open(path).wrap_err("Failed to open config path")?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .wrap_err("Failed to read work file")?;

    let file: WorkDataFile = ron::from_str(&buf).wrap_err("Failed to parse file work entries")?;
    Ok(file)
}
