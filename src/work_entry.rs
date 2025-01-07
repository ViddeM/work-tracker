use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{work_entry_id::WorkEntryId, work_entry_status::WorkEntryStatus};

#[derive(Serialize, Deserialize)]
pub struct WorkEntry {
    pub id: WorkEntryId, // Just an incremental integer.
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub status: WorkEntryStatus,
    pub children: Vec<WorkEntry>,
}

impl WorkEntry {
    pub fn new(id: WorkEntryId, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            status: WorkEntryStatus::Created,
            children: vec![],
        }
    }

    pub fn complete(&mut self) {
        self.modified_at = Utc::now();
        self.status = WorkEntryStatus::Completed;
    }

    pub fn to_printable_row(&self) -> String {
        format!(
            " {} {} {} {}",
            self.id,
            "->>".green(),
            self.name.bright_cyan(),
            self.status.get_icon(),
        )
    }

    pub fn is_completed(&self) -> bool {
        self.status == WorkEntryStatus::Completed
    }
}
