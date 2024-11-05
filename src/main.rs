use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use eyre::{Context, OptionExt};
use home::home_dir;
use work_data_file::WorkDataFile;
use work_entry::WorkEntry;
use work_entry_status::WorkEntryStatus;

pub mod work_data_file;
pub mod work_entry;
pub mod work_entry_status;

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

        /// The new description of the entry if any.
        #[arg(short, long)]
        description: Option<String>,

        /// The new status of the entry.
        #[arg(short, long)]
        status: Option<WorkEntryStatus>,
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
        Some(WorkAction::Edit {
            id,
            description,
            status,
        }) => {
            let entry = wd_file.get_entry_mut(id)?;

            if description.is_none() && status.is_none() {
                eyre::bail!("No action provided to edit the entry, please provide either description or status (or both)");
            }

            entry.description = description;

            if let Some(status) = status {
                entry.status = status;
            }

            wd_file
                .save(&config_path)
                .wrap_err("Failed to save changes")?;
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
                "{} {div} {} {div} {} {div} {} {div} {} / {}",
                id.to_string().bright_blue(),
                name.bright_green(),
                description
                    .as_ref()
                    .unwrap_or(&"<No description>".to_string())
                    .yellow(),
                status.to_colored_string(),
                created_at.to_formatted_string(),
                modified_at.to_formatted_string()
            )
        }
    };

    Ok(())
}

trait DisplayableDateTime {
    fn to_formatted_string(self) -> String;
}

impl DisplayableDateTime for DateTime<Utc> {
    fn to_formatted_string(self) -> String {
        self.format("%H:%M %d/%m %Y").to_string()
    }
}

fn get_or_create_file_file(path: &Path) -> eyre::Result<WorkDataFile> {
    if !path.exists() {
        let wd_file = WorkDataFile::default();
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
