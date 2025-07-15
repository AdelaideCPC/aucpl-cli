use std::fs::{self, File};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::from_reader;

use crate::config::Settings;
use crate::problem::run::RunnableFile;
use crate::problem::solve::solve as problem_solve;

use super::{Competitions, COMPETITIONS_FILE};

pub fn solve(
    settings: &Settings,
    problems_dir: &Path,
    comp_name: &str,
    solution_file: RunnableFile,
) -> Result<()> {
    let comp_file_path = problems_dir.join(COMPETITIONS_FILE);
    if !fs::exists(&comp_file_path)? {
        bail!("Competitions file does not exist");
    }

    let comp_file = File::open(&comp_file_path)?;
    let data: Competitions = from_reader(&comp_file)?;

    let comp_data = data
        .get(comp_name)
        .context(format!("Competition '{comp_name}' not found"))?;

    eprintln!("Generating output test cases for all problems in competition '{comp_name}'");
    for problem_name in &comp_data.problems {
        eprintln!("\nRunning for problem '{problem_name}'...");
        problem_solve(
            settings,
            problems_dir,
            problem_name.as_str(),
            &solution_file,
        )?;
    }

    Ok(())
}
