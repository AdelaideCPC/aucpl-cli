//! CLI commands and helper functions related to competitions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod add;
pub mod create;
pub mod finish;
pub mod list;
pub mod remove;
pub mod rename;
pub mod solve;
pub mod test;

pub const COMPETITIONS_FILE: &str = "competitions.json";

/// Map competition names to their data.
type Competitions = BTreeMap<String, CompetitionData>;

#[derive(Clone, Deserialize, Debug, Serialize)]
struct CompetitionData {
    pub problems: Vec<String>,
    pub finished: bool,
}
