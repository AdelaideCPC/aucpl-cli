use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_json::{from_reader, to_writer_pretty};

use crate::problem::archive::archive;

use super::{Competitions, COMPETITIONS_FILE};

pub fn finish(problems_dir: &PathBuf, comp_name: &str) -> Result<()> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path)? {
        bail!("Competitions file does not exist");
    }

    let comp_file = File::open(&comp_file_path)?;
    let mut data: Competitions = from_reader(&comp_file)?;
    let comp_data = data
        .get_mut(comp_name)
        .context(format!("Competition '{comp_name}' not found"))?;

    let comp_problems = &comp_data.problems;

    // Archive problems
    for problem in comp_problems {
        archive(problems_dir, problem)?;
        eprintln!("Archived problem {problem}");
    }

    comp_data.finished = true;
    let comp_file = File::options()
        .write(true)
        .truncate(true)
        .open(comp_file_path)?;
    to_writer_pretty(&comp_file, &data)?;

    Ok(())
}
