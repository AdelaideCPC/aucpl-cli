use std::fs::{self, File};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::{from_reader, to_writer_pretty};

use crate::problem::sync_mappings::problem_exists;

use super::{Competitions, COMPETITIONS_FILE};

pub fn add(problems_dir: &Path, comp_name: &str, problem_name: &str) -> Result<()> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path)? {
        bail!("Competitions file does not exist");
    }

    let comp_file = File::open(&comp_file_path)?;
    let mut data: Competitions = from_reader(&comp_file)?;

    let comp_data = data
        .get_mut(comp_name)
        .context(format!("Competition '{comp_name}' not found"))?;

    if !problem_exists(problems_dir, problem_name)? {
        eprintln!("The problem '{problem_name}' does not exist");
        return Ok(());
    }
    if comp_data.finished {
        eprintln!("Cannot add problems to an archived competition");
        return Ok(());
    }

    comp_data.problems.push(problem_name.to_string());
    comp_data.problems.sort_unstable();

    let comp_file = File::options()
        .write(true)
        .truncate(true)
        .open(comp_file_path)?;
    to_writer_pretty(&comp_file, &data)?;
    eprintln!("Added problem '{problem_name}' to the competition '{comp_name}'");

    Ok(())
}
