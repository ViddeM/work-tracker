use std::{fs::File, io::Write, path::Path};

use eyre::{Context, ContextCompat};
use serde::{Deserialize, Serialize};

use crate::{
    work_entry::WorkEntry,
    work_entry_id::{WorkEntryId, WorkEntryIdFull},
};

#[derive(Serialize, Deserialize)]
pub struct WorkDataFile {
    pub version: FileVersion,
    pub entries: Vec<WorkEntry>,
}

impl WorkDataFile {
    pub fn is_current(&self) -> bool {
        self.version == FileVersion::current()
    }

    pub fn add_entry(&mut self, name: String, description: Option<String>) {
        let highest_num = self
            .entries
            .iter()
            .map(|e| &e.id)
            .max()
            .map(|id| id.next())
            .unwrap_or_default();

        let new_entry = WorkEntry::new(highest_num, name, description);

        self.entries.push(new_entry);
    }

    pub fn add_child_entry(
        &mut self,
        name: String,
        description: Option<String>,
        parent: WorkEntryIdFull,
    ) {
        todo!("Not updated");
    }

    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let serialized = ron::to_string(&self).wrap_err("Failed to serialize wd_file")?;

        let mut file = File::create(path).wrap_err("Failed to open config file to save changes")?;
        file.write_all(serialized.as_bytes())
            .wrap_err("Failed to write changes to file")?;

        Ok(())
    }

    pub fn get_index_for_id(&self, id: &WorkEntryId) -> eyre::Result<usize> {
        let (current_index, _) = self
            .entries
            .iter()
            .enumerate()
            .find(|(_, entry)| &entry.id == id)
            .wrap_err("No entry with the provided ID")?;

        Ok(current_index)
    }

    pub fn get_entry(&self, id: &WorkEntryId) -> eyre::Result<&WorkEntry> {
        self.entries
            .iter()
            .find(|entry| &entry.id == id)
            .wrap_err("Failed to find entry with the provided ID")
    }

    pub fn get_entry_or_first(&self, id: Option<&WorkEntryId>) -> eyre::Result<Option<&WorkEntry>> {
        if let Some(id) = id {
            return self.get_entry(id).map(Some);
        }

        return Ok(self.entries.iter().filter(|e| !e.is_completed()).last());
    }

    pub fn get_entry_mut(&mut self, id: &WorkEntryId) -> eyre::Result<&mut WorkEntry> {
        self.entries
            .iter_mut()
            .find(|entry| &entry.id == id)
            .wrap_err("Failed to find entry with the provided ID")
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum FileVersion {
    Initial,
    Nested,
}

impl FileVersion {
    pub fn current() -> Self {
        Self::Nested
    }
}
