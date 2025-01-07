use std::{fmt::Display, str::FromStr};

use eyre::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct WorkEntryIdFull(Vec<WorkEntryId>);

impl FromStr for WorkEntryIdFull {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s
            .split(".")
            .map(|e| WorkEntryId::from_str(e))
            .collect::<eyre::Result<Vec<WorkEntryId>>>()?;

        Ok(Self(id))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct WorkEntryId(usize);

impl FromStr for WorkEntryId {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.parse::<usize>().wrap_err("Failed to parse id as usize")?;
        Ok(Self(num))
    }
}

impl Display for WorkEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WorkEntryId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl Default for WorkEntryId {
    fn default() -> Self {
        Self(Default::default())
    }
}
