use std::fs::{self, File};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::from_reader;

use super::{Competitions, COMPETITIONS_FILE};

pub fn list(problems_dir: &Path, comp_name: Option<&String>) -> Result<()> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path)? {
        bail!("Competitions file does not exist");
    }

    let comp_file = File::open(&comp_file_path)?;
    let data: Competitions = from_reader(&comp_file)?;

    match comp_name {
        Some(name) => {
            let comp_data = data
                .get(name)
                .context(format!("Competition '{name}' not found"))?;

            // TODO: Print difficulty as well
            // NOTE: More metadata can be listed once we actually store them
            eprintln!("Problems in '{name}':");
            for p in &comp_data.problems {
                eprintln!("  - {p}")
            }
            eprintln!("Total problems: {}", comp_data.problems.len());
        }
        None => {
            eprintln!("Competitions:");
            for comp in data.keys() {
                eprintln!(" - {comp}");
            }
            eprintln!("Total competitions: {}", data.keys().len());
        }
    }

    Ok(())
}
